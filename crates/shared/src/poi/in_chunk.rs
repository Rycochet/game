//! The poi in one chunk, if any: its kind, kept to land, and towns.

use super::{
    CHUNK_SIZE, ChunkPos, EDGE_MARGIN, LAND_TRIES, POI_CHANCE_PERCENT, Poi, PoiKind, PoiMap,
};
use crate::climate::Biome;
use crate::rng::Seed;
use crate::showcase;

/// Of each 1000 pois, how many are towns of each size, by a separate hash
/// so every other chunk keeps the poi it had before towns existed.
pub(super) const TOWNS_PER_THOUSAND: [(PoiKind, u64); 4] = [
    (PoiKind::Hamlet, 30),
    (PoiKind::Village, 15),
    (PoiKind::Town, 5),
    (PoiKind::City, 2),
];

/// Of each 1000 pois on land that suits one, how many are farms, by a separate
/// hash for the same reason.
pub(super) const FARMS_PER_THOUSAND: u64 = 60;

/// The same for herbalists' huts.
pub(super) const HUTS_PER_THOUSAND: u64 = 40;

/// In desert, only one chunk in this many keeps its poi (owner: a quarter as
/// many there).
const DESERT_KEEPS_ONE_IN: u64 = 4;

impl PoiMap {
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
        if self.has_gate(chunk) {
            return None;
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
        (0..LAND_TRIES)
            .find_map(|_| {
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
                    desert: self.climate.sample(x, y, height).biome() == Biome::Desert,
                })
            })
            .filter(|poi| !self.ground.past_the_ice(poi.y))
            .filter(|poi| {
                !poi.desert
                    || self
                        .seed
                        .derive("desert")
                        .at(&[chunk.x, chunk.y])
                        .is_multiple_of(DESERT_KEEPS_ONE_IN)
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
}
