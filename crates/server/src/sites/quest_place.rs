//! What quests need of where the player is: the poi they are in (a site, or a
//! town or other poi set into the wilderness), and a giver's door.

use shared::poi::{ChunkPos, Poi};
use shared::shop::beside_door;
use shared::site::BuildingLook;

use super::{Place, Visit};
use crate::ServerWorld;
use crate::connections::Client;
use crate::storage::SiteId;

/// Where the player is, as quests count it.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct QuestPlace {
    /// The poi they are in.
    pub(crate) poi: Poi,
    /// Which of its levels (0 for a poi set into the wilderness).
    pub(crate) level: u32,
    /// Whether it is the site's deepest level.
    pub(crate) deepest: bool,
}

impl Visit {
    /// The poi the player is in for quests: a site's, or the one whose cutout
    /// they stand in, in the wilderness.
    pub(crate) fn quest_place(&self) -> Option<QuestPlace> {
        match &self.place {
            Place::Site { poi, site, .. } => Some(QuestPlace {
                poi: *poi,
                level: self.index,
                deepest: self.index + 1 >= site.levels(),
            }),
            Place::Wilderness { in_poi, .. } => in_poi.map(|poi| QuestPlace {
                poi,
                level: 0,
                deepest: false,
            }),
        }
    }
}

/// Whether the player stands beside the door of building `building` of the
/// town in `town` (`shop::beside_door`), on its own level or where it is set
/// into the wilderness.
pub(crate) fn beside_giver(
    client: &Client,
    world: &mut ServerWorld,
    town: ChunkPos,
    building: u16,
) -> bool {
    let Some(visit) = &client.visit else {
        return false;
    };
    let at = visit.spot();
    if !visit.is_wilderness() {
        let here = visit.poi().is_some_and(|poi| poi.chunk == town);
        let mut buildings = visit.level.buildings.iter();
        return here && buildings.any(|b| b.look.id == building && beside_door(at, b.look.door));
    }
    let Some(mirror) = (visit.mirrors.iter()).find(|m| m.id == SiteId::Poi(town)) else {
        return false;
    };
    let offset = mirror.offset;
    let Some(chunk) = super::wild_chunk(world, town) else {
        return false;
    };
    let mut buildings = chunk.town.iter();
    buildings.any(|b| {
        let (x, y) = b.look.door;
        b.look.id == building && beside_door(at, (x - offset.0, y - offset.1))
    })
}

/// A town's buildings as its first level has them: what its quests' givers
/// are (`shared::quest::givers`).
pub(crate) fn town_buildings(world: &mut ServerWorld, town: &Poi) -> Vec<BuildingLook> {
    let Some(site) = super::wild::site_as_saved(world, town) else {
        return Vec::new();
    };
    let first = site.level(0);
    first.buildings.iter().map(|b| b.look.clone()).collect()
}
