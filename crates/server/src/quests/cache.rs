//! Each town's quests, worked out once and kept (`shared::quest` takes
//! milliseconds a town): its normal quests, and its dailies by day. Not
//! saved: the same seed always makes the same quests.

use std::collections::HashMap;
use std::sync::Arc;

use shared::poi::{ChunkPos, Poi};
use shared::quest::{Quest, QuestId, QuestWorld, Slot, daily_quest, town_quests};
use shared::road::Roads;
use shared::site::BuildingLook;
use shared::world::WorldParams;

use crate::ServerWorld;
use crate::sites::town_buildings;

/// Towns kept before all are let go.
const MOST_TOWNS: usize = 256;

/// One world's towns' quests (`ServerWorld::quests`).
pub(crate) struct TownQuests {
    /// Which towns are joined by road (only the links are read, which are
    /// quick: no road is laid).
    roads: Roads,
    towns: HashMap<ChunkPos, Town>,
    dailies: HashMap<(ChunkPos, u16, i64), Option<Arc<Quest>>>,
}

/// A town's givers' buildings and its normal quests.
#[derive(Clone)]
pub(crate) struct Town {
    pub(crate) poi: Poi,
    pub(crate) buildings: Arc<Vec<BuildingLook>>,
    pub(crate) quests: Arc<Vec<Quest>>,
}

impl TownQuests {
    pub(crate) fn new(params: &WorldParams) -> Self {
        Self {
            roads: Roads::new(params),
            towns: HashMap::new(),
            dailies: HashMap::new(),
        }
    }
}

/// The town in `chunk`, with its quests, or `None` if there is no town there.
pub(crate) fn town(world: &mut ServerWorld, chunk: ChunkPos) -> Option<Town> {
    if let Some(town) = world.quests.towns.get(&chunk) {
        return Some(town.clone());
    }
    let poi = world
        .pois
        .in_chunk(chunk)
        .filter(|poi| poi.kind.is_town())?;
    let buildings = town_buildings(world, &poi);
    let ServerWorld {
        pois,
        ground,
        quests,
        ..
    } = world;
    let name_of =
        |poi: &Poi| shared::place_name::place_name(poi.kind, poi.seed, ground.metres(poi.x, poi.y));
    let quest_world = QuestWorld {
        pois,
        roads: &quests.roads,
        name_of: &name_of,
    };
    let made = Town {
        poi,
        quests: Arc::new(town_quests(&quest_world, &poi, &buildings)),
        buildings: Arc::new(buildings),
    };
    if quests.towns.len() >= MOST_TOWNS {
        quests.towns.clear();
    }
    quests.towns.insert(chunk, made.clone());
    Some(made)
}

/// The daily quest a town's giver offers on a UTC day, if they give dailies.
pub(crate) fn daily(
    world: &mut ServerWorld,
    chunk: ChunkPos,
    building: u16,
    day: i64,
) -> Option<Arc<Quest>> {
    let key = (chunk, building, day);
    if let Some(found) = world.quests.dailies.get(&key) {
        return found.clone();
    }
    let town = town(world, chunk)?;
    let ServerWorld {
        pois,
        ground,
        quests,
        ..
    } = world;
    let name_of =
        |poi: &Poi| shared::place_name::place_name(poi.kind, poi.seed, ground.metres(poi.x, poi.y));
    let quest_world = QuestWorld {
        pois,
        roads: &quests.roads,
        name_of: &name_of,
    };
    let made = daily_quest(&quest_world, &town.poi, &town.buildings, building, day).map(Arc::new);
    if quests.dailies.len() >= MOST_TOWNS {
        quests.dailies.clear();
    }
    quests.dailies.insert(key, made.clone());
    made
}

/// A quest by its id, made again from its town's seed.
pub(crate) fn quest(world: &mut ServerWorld, id: QuestId) -> Option<Quest> {
    if let Slot::Daily { building, day } = id.slot {
        return daily(world, id.town, building, day).map(|quest| (*quest).clone());
    }
    let town = town(world, id.town)?;
    town.quests.iter().find(|quest| quest.id == id).cloned()
}
