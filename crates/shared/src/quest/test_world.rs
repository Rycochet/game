//! The client's fixed test world, its towns and their buildings, for tests.

use super::QuestWorld;
use crate::place_name::place_name;
use crate::poi::{ChunkPos, Poi, PoiMap};
use crate::rng::Seed;
use crate::road::Roads;
use crate::site::{BuildingLook, Site};
use crate::world::WorldParams;

pub(super) struct TestWorld {
    pub pois: PoiMap,
    pub roads: Roads,
}

impl TestWorld {
    pub fn new() -> Self {
        let params = WorldParams::from_seed(Seed::new(12345));
        Self {
            pois: PoiMap::new(&params, params.seed.derive("pois")),
            roads: Roads::new(&params),
        }
    }

    /// The generator's view of it (names as if every place were at sea level).
    pub fn view(&self) -> QuestWorld<'_> {
        QuestWorld {
            pois: &self.pois,
            roads: &self.roads,
            name_of: &name_of,
            moons: 2,
        }
    }

    /// Every town in chunks `0..side` × `0..side`, by chunk.
    pub fn towns(&self, side: i64) -> Vec<Poi> {
        (0..side)
            .flat_map(|y| (0..side).map(move |x| ChunkPos { x, y }))
            .filter_map(|chunk| self.pois.in_chunk(chunk))
            .filter(|poi| poi.kind.is_town())
            .collect()
    }
}

fn name_of(poi: &Poi) -> Option<String> {
    place_name(poi.kind, poi.seed, 0)
}

/// A town's buildings, as the server has them from its site.
pub(super) fn buildings(town: &Poi) -> Vec<BuildingLook> {
    (Site::new(town.kind, town.seed)
        .level(0)
        .buildings
        .into_iter())
    .map(|building| building.look)
    .collect()
}
