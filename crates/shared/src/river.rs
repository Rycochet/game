//! Rivers on the world map. For now only something to look at: they change
//! nothing about walking, routes or pois.
//!
//! Like pois, a river comes from its chunk's position alone: some chunks
//! have a source, and water runs downhill from it until it reaches the sea or
//! a low point, where it ends. Each is traced on its own, so the order chunks
//! are looked at doesn't matter; rivers merge because downhill leads them into
//! the same valleys. See `docs/context/shared/river.md`.

use crate::climate::{Biome, Climate};
use crate::poi::{CHUNK_SIZE, ChunkPos, EDGE_MARGIN, LAND_TRIES};
use crate::rng::Seed;
use crate::surface::Surface;
use crate::world::WorldParams;

/// The highest chance of a chunk having a source, in the wettest biomes.
const MOST_LIKELY_PERCENT: u64 = 30;

/// The chance of a chunk having a source, by what grows there: dry biomes few,
/// wet ones many.
fn chance_percent(biome: Biome) -> u64 {
    match biome {
        Biome::Desert => 3,
        Biome::Tundra => 6,
        Biome::Grassland => 8,
        Biome::Savanna => 10,
        Biome::Woodland => 14,
        Biome::BorealForest => 16,
        Biome::SeasonalForest => 20,
        Biome::TemperateRainforest | Biome::TropicalRainforest => MOST_LIKELY_PERCENT,
    }
}

/// The longest a river runs, in steps. Bounds how far away a source can
/// matter: a chunk's rivers come from within this distance.
pub const MAX_LENGTH: usize = 256;

/// The least a point's random pull scales its slope by (the most is this plus
/// 1): smaller makes rivers wander more.
const MIN_PULL: f64 = 0.15;

/// A river's course, from its source to where it ends.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct River {
    /// World grid points, one step apart. `x` starts within the first planet
    /// width and carries on past its edges without wrapping.
    pub points: Vec<(i64, i64)>,
    /// Whether it ended at the sea, rather than at a low point or its length.
    pub reaches_sea: bool,
}

/// Finds the rivers of one world. Uses the public terrain seed, since the
/// client draws them itself.
pub struct Rivers {
    seed: Seed,
    surface: Surface,
    climate: Climate,
    width: i64,
    /// Sources are above this: high tide.
    land_above: f64,
    /// Rivers end when the ground is at this: low tide, so they always meet
    /// the water.
    sea_at: f64,
}

impl Rivers {
    pub fn new(world: &WorldParams) -> Self {
        let surface = Surface::new(world);
        Self {
            seed: world.seed.derive("rivers"),
            climate: Climate::new(world),
            width: surface.width(),
            surface,
            land_above: world.high_water(),
            sea_at: 2.0 * world.sea_level - world.high_water(),
        }
    }

    /// Chunk columns round the planet (as `PoiMap::columns`).
    pub fn columns(&self) -> i64 {
        (self.width / CHUNK_SIZE).max(1)
    }

    /// Grid units around the equator; `x` wraps at this value.
    pub fn width(&self) -> i64 {
        self.width
    }

    /// The river starting in a chunk, if it has one.
    pub fn in_chunk(&self, chunk: ChunkPos) -> Option<River> {
        if crate::showcase::chunk_in_plain(chunk) {
            return None;
        }
        let source = self.source(chunk)?;
        Some(self.trace(source))
    }

    /// Where a chunk's river starts: the highest of a few random spots that
    /// is dry land, so rivers tend to start up in the hills.
    fn source(&self, chunk: ChunkPos) -> Option<(i64, i64)> {
        let column = chunk.x.rem_euclid(self.columns());
        let mut rng = Seed::new(self.seed.at(&[column, chunk.y])).rng();
        let roll = rng.range_u64(0, 100);
        // Most chunks can be ruled out without looking at the ground.
        if roll >= MOST_LIKELY_PERCENT {
            return None;
        }
        // The last column is wider (see `columns`).
        let west = column * CHUNK_SIZE;
        let east = if column == self.columns() - 1 {
            self.width
        } else {
            west + CHUNK_SIZE
        };
        // Worlds narrower than the margins can't place anything.
        let across = (east - west - 2 * EDGE_MARGIN).max(1) as u64;
        let north = chunk.y * CHUNK_SIZE;
        let mut best: Option<(f64, i64, i64)> = None;
        for _ in 0..LAND_TRIES {
            let x = west + EDGE_MARGIN + rng.range_u64(0, across) as i64;
            let y = north
                + EDGE_MARGIN
                + rng.range_u64(0, (CHUNK_SIZE - 2 * EDGE_MARGIN) as u64) as i64;
            let height = self.surface.sample(x, y).height;
            if height > self.land_above && best.is_none_or(|(highest, ..)| height > highest) {
                best = Some((height, x, y));
            }
        }
        let (_, x, y) = best?;
        // The biome of the region, not cooled by the height of the source
        // (which is usually high up).
        let region = self.climate.sample(x, y, 0.0).biome();
        (roll < chance_percent(region)).then_some((x, y))
    }

    /// Follows the steepest way down from `source`. Goes by the terrain's
    /// broad height, so small bumps don't end a river early.
    fn trace(&self, source: (i64, i64)) -> River {
        /// The 8 neighbours; diagonals are compared by slope, so their longer
        /// distance counts. The order settles exact ties.
        const STEPS: [(i64, i64); 8] = [
            (0, -1),
            (1, 0),
            (0, 1),
            (-1, 0),
            (1, -1),
            (1, 1),
            (-1, 1),
            (-1, -1),
        ];
        let mut points = vec![source];
        let (mut x, mut y) = source;
        let mut here = self.surface.broad_height(x, y);
        while points.len() < MAX_LENGTH {
            if self.surface.sample(x, y).height <= self.sea_at {
                return River {
                    points,
                    reaches_sea: true,
                };
            }
            let mut best: Option<(f64, f64, i64, i64)> = None;
            for (dx, dy) in STEPS {
                let there = self.surface.broad_height(x + dx, y + dy);
                let distance = if dx != 0 && dy != 0 {
                    core::f64::consts::SQRT_2
                } else {
                    1.0
                };
                let slope = (here - there) / distance;
                if slope <= 0.0 {
                    continue;
                }
                // Each point has its own pull, so courses wander. It belongs
                // to the place, not the river, so rivers that meet stay
                // together.
                let pull = self.seed.at(&[(x + dx).rem_euclid(self.width), y + dy]) % 1024;
                let score = slope * (MIN_PULL + pull as f64 / 1024.0);
                if best.is_none_or(|(highest, ..)| score > highest) {
                    best = Some((score, there, x + dx, y + dy));
                }
            }
            // A low point: the river ends here.
            let Some((_, there, next_x, next_y)) = best else {
                break;
            };
            (x, y, here) = (next_x, next_y, there);
            points.push((x, y));
        }
        River {
            points,
            reaches_sea: false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn world() -> WorldParams {
        WorldParams::from_seed(Seed::new(12345))
    }

    fn all_rivers(rivers: &Rivers) -> Vec<River> {
        let mut found = Vec::new();
        for y in -40..40 {
            for x in 0..80 {
                found.extend(rivers.in_chunk(ChunkPos { x, y }));
            }
        }
        found
    }

    #[test]
    fn about_the_right_share_of_chunks_have_rivers() {
        let rivers = Rivers::new(&world());
        let count = all_rivers(&rivers).len();
        // 6400 chunks at 3-30%, less those with no land.
        assert!((300..=1900).contains(&count), "{count}");
    }

    #[test]
    fn rivers_run_downhill_one_step_at_a_time() {
        let world = world();
        let (rivers, surface) = (Rivers::new(&world), Surface::new(&world));
        let all = all_rivers(&rivers);
        for river in &all {
            for pair in river.points.windows(2) {
                let ((ax, ay), (bx, by)) = (pair[0], pair[1]);
                assert!((ax - bx).abs() <= 1 && (ay - by).abs() <= 1);
                assert!(surface.broad_height(bx, by) < surface.broad_height(ax, ay));
            }
        }
        assert!(all.iter().any(|river| river.reaches_sea));
        assert!(all.iter().any(|river| !river.reaches_sea));
    }

    #[test]
    fn rivers_stop_at_the_sea() {
        let world = world();
        let (rivers, surface) = (Rivers::new(&world), Surface::new(&world));
        let low_tide = 2.0 * world.sea_level - world.high_water();
        for river in all_rivers(&rivers) {
            let (_, before_last) = river.points.split_last().unwrap();
            for &(x, y) in before_last {
                assert!(surface.sample(x, y).height > low_tide);
            }
        }
    }

    #[test]
    fn wet_biomes_have_more_rivers_than_dry_ones() {
        assert!(chance_percent(Biome::Desert) < chance_percent(Biome::TropicalRainforest));
        assert!(
            Biome::ALL
                .iter()
                .all(|&b| chance_percent(b) <= MOST_LIKELY_PERCENT)
        );
    }

    #[test]
    fn the_same_chunk_gives_the_same_river_after_a_wrap() {
        let rivers = Rivers::new(&world());
        let chunk = (0..80)
            .flat_map(|x| (-40..40).map(move |y| ChunkPos { x, y }))
            .find(|&chunk| rivers.in_chunk(chunk).is_some())
            .unwrap();
        let wrapped = ChunkPos {
            x: chunk.x + rivers.columns(),
            ..chunk
        };
        assert_eq!(rivers.in_chunk(chunk), rivers.in_chunk(wrapped));
    }
}
