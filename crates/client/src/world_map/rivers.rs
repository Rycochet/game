//! Rivers near the player, kept ready to draw: traced once from their chunk,
//! then smoothed, with a width at every point. `terrain::build_rivers` turns
//! them into the mesh.

use std::collections::BTreeMap;

use bevy::prelude::*;
use shared::climate::Climate;
use shared::poi::{CHUNK_SIZE, ChunkPos};
use shared::river::{MAX_LENGTH, River, Rivers};
use shared::surface::Surface;

/// World map grid units per screen pixel in the starting view (a 400-point
/// window is about 1250 pixels wide), so widths can be given in pixels.
const UNITS_PER_PIXEL: f32 = 0.32;

/// Width at the source and the widest a river gets, in pixels at the starting
/// zoom (the owner's figures). Width comes only from how far it has run.
const SOURCE_PIXELS: f32 = 1.0;
const WIDEST_PIXELS: f32 = 4.0;

/// Points a river runs for each extra pixel of width: a chunk (the owner's
/// figure), so short rivers stay thin wherever they end.
const POINTS_PER_PIXEL: f32 = CHUNK_SIZE as f32;

/// Points either side averaged to take the grid's corners out of a course.
const SMOOTHING: usize = 4;

/// Chunks each way from the player a source can be and still reach the window.
const CHUNK_REACH: i64 = (MAX_LENGTH as i64 + super::terrain::REACH) / CHUNK_SIZE + 1;

/// Woods are cut back round rivers, so they run on the ground: nothing grows
/// within `CLEARED` grid units of the middle of one, and woods are back to full
/// by `CLEARED_TO`. Wider than the river, as grid points are a unit apart.
const CLEARED: f32 = 1.2;
const CLEARED_TO: f32 = 4.0;

/// Side of the wrapping store of clearings. It is rebuilt each time the player
/// enters a chunk, so it must hold what the window can need from anywhere in
/// that chunk.
const CLEARING_SIDE: i64 = 768;
const _: () = assert!(CLEARING_SIDE > 2 * (super::terrain::REACH + CHUNK_SIZE + 2));

/// A river ready to draw.
pub struct DrawnRiver {
    /// Where it starts, as a world grid position.
    pub source: (i64, i64),
    /// Smoothed course, relative to the source.
    pub points: Vec<Vec2>,
    /// Width at each point, in grid units.
    pub widths: Vec<f32>,
    /// Temperature at each point, which decides if it is frozen.
    pub temperatures: Vec<f64>,
}

impl DrawnRiver {
    fn new(river: &River, climate: &Climate, surface: &Surface) -> Self {
        let source = river.points[0];
        let last = river.points.len() - 1;
        let relative = |i: usize| {
            let (x, y) = river.points[i];
            Vec2::new((x - source.0) as f32, (y - source.1) as f32)
        };
        let points = (0..=last)
            .map(|i| {
                // A shorter window near the ends, so they stay where they are.
                let reach = SMOOTHING.min(i).min(last - i);
                let total: Vec2 = (i - reach..=i + reach).map(relative).sum();
                total / (2 * reach + 1) as f32
            })
            .collect();
        let widths = (0..=last)
            .map(|i| {
                let extra = i as f32 / POINTS_PER_PIXEL;
                (SOURCE_PIXELS + extra).min(WIDEST_PIXELS) * UNITS_PER_PIXEL
            })
            .collect();
        let temperatures = river
            .points
            .iter()
            .map(|&(x, y)| {
                climate
                    .sample(x, y, surface.sample(x, y).height)
                    .temperature
            })
            .collect();
        Self {
            source,
            points,
            widths,
            temperatures,
        }
    }
}

/// The chunks around the player and their rivers. A chunk stays here until the
/// player has gone well away, since tracing is the slow part.
#[derive(Resource, Default)]
pub struct KnownRivers {
    chunks: BTreeMap<ChunkPos, Option<DrawnRiver>>,
    /// The chunk last made sure of, so most steps do nothing.
    around: Option<ChunkPos>,
    /// How much of the woods grow at each point near the player, `0..=1`.
    /// Empty until the first update.
    open: Vec<f32>,
}

impl KnownRivers {
    /// Makes sure every chunk whose river could reach the window is known.
    pub fn update(
        &mut self,
        rivers: &Rivers,
        climate: &Climate,
        surface: &Surface,
        (x, y): (i64, i64),
    ) {
        let columns = rivers.columns();
        let here = ChunkPos {
            x: x.rem_euclid(rivers.width()).div_euclid(CHUNK_SIZE),
            y: y.div_euclid(CHUNK_SIZE),
        };
        if self.around == Some(here) {
            return;
        }
        self.around = Some(here);
        for dy in -CHUNK_REACH..=CHUNK_REACH {
            for dx in -CHUNK_REACH..=CHUNK_REACH {
                let chunk = ChunkPos {
                    x: (here.x + dx).rem_euclid(columns),
                    y: here.y + dy,
                };
                self.chunks.entry(chunk).or_insert_with(|| {
                    let river = rivers.in_chunk(chunk);
                    river.map(|river| DrawnRiver::new(&river, climate, surface))
                });
            }
        }
        // Forget chunks a good way beyond that (east-west, measured round the
        // planet).
        let keep = CHUNK_REACH + 4;
        self.chunks.retain(|chunk, _| {
            let east = (chunk.x - here.x).rem_euclid(columns);
            east.min(columns - east) <= keep && (chunk.y - here.y).abs() <= keep
        });
        self.clear_round_rivers((x, y), rivers.width());
    }

    /// Works out `open` for the points around `centre`.
    fn clear_round_rivers(&mut self, centre: (i64, i64), world_width: i64) {
        let mut open = std::mem::take(&mut self.open);
        open.clear();
        open.resize((CLEARING_SIDE * CLEARING_SIDE) as usize, 1.0);
        let limit = CLEARING_SIDE / 2 - 1;
        let reach = CLEARED_TO.ceil() as i64;
        for river in self.rivers() {
            // The way round the planet that is nearest the player.
            let mut east = (river.source.0 - centre.0).rem_euclid(world_width);
            if east > world_width / 2 {
                east -= world_width;
            }
            let (x0, y0) = (east as f32, (river.source.1 - centre.1) as f32);
            for point in &river.points {
                let (px, py) = (point.x + x0, point.y + y0);
                let (nx, ny) = (px.round() as i64, py.round() as i64);
                for dy in ny - reach..=ny + reach {
                    for dx in nx - reach..=nx + reach {
                        if dx.abs() > limit || dy.abs() > limit {
                            continue;
                        }
                        let distance = (dx as f32 - px).hypot(dy as f32 - py);
                        let t = ((distance - CLEARED) / (CLEARED_TO - CLEARED)).clamp(0.0, 1.0);
                        let slot = clearing_slot(centre.0 + dx, centre.1 + dy);
                        open[slot] = open[slot].min(t * t * (3.0 - 2.0 * t));
                    }
                }
            }
        }
        self.open = open;
    }

    /// How much of the woods grow at a point near the player: 0 beside a river,
    /// 1 away from any.
    pub fn open_at(&self, x: i64, z: i64) -> f32 {
        self.open.get(clearing_slot(x, z)).copied().unwrap_or(1.0)
    }

    pub fn rivers(&self) -> impl Iterator<Item = &DrawnRiver> {
        self.chunks.values().flatten()
    }
}

fn clearing_slot(x: i64, z: i64) -> usize {
    (z.rem_euclid(CLEARING_SIDE) * CLEARING_SIDE + x.rem_euclid(CLEARING_SIDE)) as usize
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn woods_are_cleared_beside_a_river_and_back_away_from_it() {
        let mut known = KnownRivers::default();
        known.chunks.insert(
            ChunkPos { x: 0, y: 0 },
            Some(DrawnRiver {
                source: (100, 50),
                points: (0..20).map(|i| Vec2::new(i as f32, 0.0)).collect(),
                widths: vec![1.0; 20],
                temperatures: vec![0.5; 20],
            }),
        );
        assert_eq!(known.open_at(105, 50), 1.0);
        known.clear_round_rivers((90, 40), 100_000);
        assert_eq!(known.open_at(105, 50), 0.0);
        let part_way = known.open_at(105, 53);
        assert!(part_way > 0.0 && part_way < 1.0, "{part_way}");
        assert_eq!(known.open_at(105, 60), 1.0);
    }
}
