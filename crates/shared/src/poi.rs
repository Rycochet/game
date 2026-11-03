//! World map pois: at most one per chunk of the world grid, such as a
//! cave, a house, a castle or a crypt. Each one leads to a map (a dungeon,
//! building or town) generated from the poi's own seed.
//!
//! Everything about a poi comes from its chunk's position alone: whether
//! there is one, what it is, exactly where it stands in the chunk, and the
//! seed its map is generated from. Pois stand only on dry land (above
//! high tide), so a chunk that is all sea has none.

use std::collections::BTreeSet;

use serde::{Deserialize, Serialize};

use crate::climate::{Biome, Climate};
use crate::rng::Seed;
use crate::showcase;
use crate::surface::Surface;
use crate::world::WorldParams;

/// Chunks are this many world grid points (about 1 km each) on a side.
pub const CHUNK_SIZE: i64 = 64;

/// Pois stand at least this far inside their chunk's edges, so two never
/// come closer than twice this (enough for their map tokens not to overlap).
pub const EDGE_MARGIN: i64 = 8;

/// Percentage of chunks that have a poi (where there is land for it).
pub const POI_CHANCE_PERCENT: u64 = 80;

/// Spots tried in a chunk to find dry land before giving up: a chunk that is
/// mostly sea may have no poi.
pub const LAND_TRIES: u32 = 8;

/// What a poi is. The order matches the Dungeon Odyssey terrain tiles
/// `l2_terrain066` to `l2_terrain099` the client shows them with.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum PoiKind {
    Boulders,
    Ruins,
    GrassyCave,
    EarthCave,
    RockCave,
    IceCave,
    ThatchedHut,
    AdobeHouse,
    AdobeStore,
    StiltHut,
    RedRoofCottage,
    GreyRoofCottage,
    StoneCottage,
    Warehouse,
    GreenStiltHut,
    StoneTower,
    WoodenTower,
    RedTent,
    BlueTent,
    GreenTent,
    Castle,
    Fort,
    IceShelter,
    SnowyCottage,
    StonePyramid,
    SandPyramid,
    ArcheryRange,
    Barrel,
    Barrels,
    Crypt,
    MossyCrypt,
    BloodyCrypt,
    Grave,
    Mausoleum,
    Hamlet,
    Village,
    Town,
    City,
    Farm,
    HerbalistHut,
}

impl PoiKind {
    pub const ALL: [Self; 40] = [
        Self::Boulders,
        Self::Ruins,
        Self::GrassyCave,
        Self::EarthCave,
        Self::RockCave,
        Self::IceCave,
        Self::ThatchedHut,
        Self::AdobeHouse,
        Self::AdobeStore,
        Self::StiltHut,
        Self::RedRoofCottage,
        Self::GreyRoofCottage,
        Self::StoneCottage,
        Self::Warehouse,
        Self::GreenStiltHut,
        Self::StoneTower,
        Self::WoodenTower,
        Self::RedTent,
        Self::BlueTent,
        Self::GreenTent,
        Self::Castle,
        Self::Fort,
        Self::IceShelter,
        Self::SnowyCottage,
        Self::StonePyramid,
        Self::SandPyramid,
        Self::ArcheryRange,
        Self::Barrel,
        Self::Barrels,
        Self::Crypt,
        Self::MossyCrypt,
        Self::BloodyCrypt,
        Self::Grave,
        Self::Mausoleum,
        Self::Hamlet,
        Self::Village,
        Self::Town,
        Self::City,
        Self::Farm,
        Self::HerbalistHut,
    ];

    /// How many of `ALL` come from the chunk's plain roll: the rest have their
    /// own (`PoiMap::town_at`, `PoiMap::kept_to_land`), so they can be rarer, or
    /// keep to the land that suits them, and every other chunk keeps its poi.
    const ROLLED: u64 = 34;

    /// Whether it is a town of any size.
    pub fn is_town(self) -> bool {
        matches!(self, Self::Hamlet | Self::Village | Self::Town | Self::City)
    }
}

/// Of each 1000 pois, how many are towns of each size, by a separate hash
/// so every other chunk keeps the poi it had before towns existed.
const TOWNS_PER_THOUSAND: [(PoiKind, u64); 4] = [
    (PoiKind::Hamlet, 30),
    (PoiKind::Village, 15),
    (PoiKind::Town, 5),
    (PoiKind::City, 2),
];

/// Of each 1000 pois on land that suits one, how many are farms, by a separate
/// hash for the same reason.
const FARMS_PER_THOUSAND: u64 = 60;

/// The same for herbalists' huts.
const HUTS_PER_THOUSAND: u64 = 40;

/// A chunk's position: chunk `(x, y)` covers world grid points from
/// `(x × CHUNK_SIZE, y × CHUNK_SIZE)`. East–west it wraps round the planet,
/// so `x` is always `0..PoiMap::columns()`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct ChunkPos {
    pub x: i64,
    pub y: i64,
}

/// One poi on the world map.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Poi {
    pub kind: PoiKind,
    /// The chunk it is in.
    pub chunk: ChunkPos,
    /// Its world grid position. `x` is in the frame the caller asked about,
    /// which may be a whole planet width east or west of the chunk.
    pub x: i64,
    pub y: i64,
    /// What its map (dungeon, building, town) is generated from.
    pub seed: Seed,
}

/// Finds the pois of one world.
pub struct PoiMap {
    seed: Seed,
    /// The world's width in grid points (`Surface::width`): `x` wraps at this.
    width: i64,
    /// Its own copy of the surface, to keep pois on land.
    surface: Surface,
    /// What grows where, as farms only stand where crops would.
    climate: Climate,
    /// Pois stand only where the ground is above this: high tide.
    land_above: f64,
}

impl PoiMap {
    /// `seed` is the world's secret pois seed (`WorldSeed::pois`):
    /// the terrain is public, but where pois are and what is inside
    /// them isn't.
    pub fn new(world: &WorldParams, seed: Seed) -> Self {
        Self::with_water(world, seed, world.high_water())
    }

    /// As `new`, with pois only where the ground is above `land_above`.
    fn with_water(world: &WorldParams, seed: Seed, land_above: f64) -> Self {
        let surface = Surface::new(world);
        Self {
            seed,
            width: surface.width(),
            surface,
            climate: Climate::new(world),
            land_above,
        }
    }

    /// Chunk columns round the planet. The width is rarely a whole number of
    /// chunks, so the last column is wider, taking up the remainder.
    pub fn columns(&self) -> i64 {
        (self.width / CHUNK_SIZE).max(1)
    }

    /// The chunk containing world grid point `(x, y)`.
    pub fn chunk_at(&self, x: i64, y: i64) -> ChunkPos {
        ChunkPos {
            x: (x.rem_euclid(self.width) / CHUNK_SIZE).min(self.columns() - 1),
            y: y.div_euclid(CHUNK_SIZE),
        }
    }

    /// The poi in a chunk, if it has one. Its `x` is within the first
    /// planet width (`0..width`).
    pub fn in_chunk(&self, chunk: ChunkPos) -> Option<Poi> {
        let chunk = ChunkPos {
            x: chunk.x.rem_euclid(self.columns()),
            y: chunk.y,
        };
        if showcase::chunk_in_plain(chunk) {
            return showcase::poi_at(chunk);
        }
        let mut rng = Seed::new(self.seed.at(&[chunk.x, chunk.y])).rng();
        if rng.range_u64(0, 100) >= POI_CHANCE_PERCENT {
            return None;
        }
        let kind = PoiKind::ALL[rng.range_u64(0, PoiKind::ROLLED) as usize];
        let kind = self.town_at(chunk).unwrap_or(kind);
        let seed = Seed::new(rng.next_u64());
        // The last column is wider (see `columns`).
        let west = chunk.x * CHUNK_SIZE;
        let east = if chunk.x == self.columns() - 1 {
            self.width
        } else {
            west + CHUNK_SIZE
        };
        let north = chunk.y * CHUNK_SIZE;
        // The first of a few random spots that is dry land.
        (0..LAND_TRIES).find_map(|_| {
            let x = west
                + EDGE_MARGIN
                + rng.range_u64(0, (east - west - 2 * EDGE_MARGIN) as u64) as i64;
            let y = north
                + EDGE_MARGIN
                + rng.range_u64(0, (CHUNK_SIZE - 2 * EDGE_MARGIN) as u64) as i64;
            let height = self.surface.sample(x, y).height;
            (height > self.land_above).then(|| Poi {
                kind: if kind.is_town() {
                    kind
                } else {
                    self.kept_to_land(chunk, x, y, height).unwrap_or(kind)
                },
                chunk,
                x,
                y,
                seed,
            })
        })
    }

    /// What a chunk's poi, standing here, is instead of what it rolled, if
    /// anything: a farm on open land, a herbalist's hut in the woods.
    fn kept_to_land(&self, chunk: ChunkPos, x: i64, y: i64, height: f64) -> Option<PoiKind> {
        let roll = |name: &str| self.seed.derive(name).at(&[chunk.x, chunk.y]) % 1000;
        let (farm, hut) = (
            roll("farms") < FARMS_PER_THOUSAND,
            roll("huts") < HUTS_PER_THOUSAND,
        );
        if !farm && !hut {
            return None;
        }
        match self.climate.sample(x, y, height).biome() {
            Biome::Grassland | Biome::Savanna if farm => Some(PoiKind::Farm),
            Biome::Woodland | Biome::SeasonalForest if farm => Some(PoiKind::Farm),
            Biome::BorealForest
            | Biome::Woodland
            | Biome::SeasonalForest
            | Biome::TemperateRainforest
            | Biome::TropicalRainforest
                if hut =>
            {
                Some(PoiKind::HerbalistHut)
            }
            _ => None,
        }
    }

    /// The town a chunk with a poi has instead of what it rolled, if any.
    fn town_at(&self, chunk: ChunkPos) -> Option<PoiKind> {
        let mut left = self.seed.derive("towns").at(&[chunk.x, chunk.y]) % 1000;
        TOWNS_PER_THOUSAND.iter().find_map(|&(kind, count)| {
            if left < count {
                return Some(kind);
            }
            left -= count;
            None
        })
    }

    /// Every chunk with any point within `min..=max` (world grid points, in
    /// any planet-width frame), each once, sorted.
    pub fn chunks_in_area(&self, min: (i64, i64), max: (i64, i64)) -> Vec<ChunkPos> {
        let mut chunks = BTreeSet::new();
        // An area wider than the planet covers every column.
        let (west, east) = if max.0 - min.0 >= self.width {
            (0, self.width - 1)
        } else {
            (min.0, max.0)
        };
        let (first, last) = (self.chunk_at(west, min.1), self.chunk_at(east, max.1));
        for cy in first.y..=last.y {
            // Across the seam at x = 0, the columns wrap round.
            let mut cx = first.x;
            loop {
                chunks.insert(ChunkPos { x: cx, y: cy });
                if cx == last.x {
                    break;
                }
                cx = (cx + 1) % self.columns();
            }
        }
        chunks.into_iter().collect()
    }

    /// Every poi whose position is within `min..=max` (world grid
    /// points, in any planet-width frame: the pois are given in the same
    /// one), sorted by position.
    pub fn in_area(&self, min: (i64, i64), max: (i64, i64)) -> Vec<Poi> {
        let mut found = Vec::new();
        // Each copy of the planet (a whole width east or west) the area
        // overlaps, then each chunk that might have a poi inside it.
        for copy in min.0.div_euclid(self.width)..=max.0.div_euclid(self.width) {
            let shift = copy * self.width;
            let west = (min.0 - shift).max(0);
            let east = (max.0 - shift).min(self.width - 1);
            if west > east {
                continue;
            }
            let (first, last) = (self.chunk_at(west, min.1), self.chunk_at(east, max.1));
            for cy in first.y..=last.y {
                for cx in first.x..=last.x {
                    let Some(mut poi) = self.in_chunk(ChunkPos { x: cx, y: cy }) else {
                        continue;
                    };
                    poi.x += shift;
                    if (min.0..=max.0).contains(&poi.x) && (min.1..=max.1).contains(&poi.y) {
                        found.push(poi);
                    }
                }
            }
        }
        found.sort_by_key(|poi| (poi.y, poi.x));
        found
    }
}

#[cfg(test)]
mod tests;
