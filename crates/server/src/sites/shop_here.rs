//! Finding a shop by its keeper on the player's level: in a town, or in a
//! town set into the wilderness, whose shops are the town's own.

use shared::id::Id;
use shared::poi::Poi;
use shared::shop::DOOR_REACH;
use shared::site::TownBuilding;

use crate::ServerWorld;
use crate::connections::Client;
use crate::storage::SiteId;

/// A shop the player's level has, as `shop_of` finds it.
pub(super) struct ShopHere {
    /// The site and level whose save keeps its keeper and stock.
    pub(super) id: SiteId,
    pub(super) index: u32,
    /// The poi the site is behind.
    pub(super) poi: Poi,
    /// The building, where its town has it.
    pub(super) building: TownBuilding,
    /// How many buildings of its kind its town has.
    pub(super) of_kind: u32,
    /// Its door's cell on the player's level.
    pub(super) door: (i64, i64),
}

impl ShopHere {
    /// Whether the player stands within `DOOR_REACH` of its door.
    pub(super) fn in_reach(&self, (x, y): (i64, i64)) -> bool {
        let (door_x, door_y) = self.door;
        (x - door_x).abs().max((y - door_y).abs()) <= DOOR_REACH
    }
}

/// The shop `with` keeps on the player's level, if they keep one there.
pub(super) fn shop_of(client: &Client, world: &mut ServerWorld, with: Id) -> Option<ShopHere> {
    let visit = client.visit.as_ref()?;
    if !visit.is_wilderness() {
        let (id, index) = (visit.id, visit.index);
        let building = keeps(world, id, index, with)?;
        let town = &visit.level.buildings;
        let found = town.iter().find(|b| b.look.id == building)?;
        return Some(ShopHere {
            id,
            index,
            poi: *visit.poi()?,
            building: found.clone(),
            of_kind: of_kind(town, found),
            door: found.look.door,
        });
    }
    // In the wilderness, the towns set into the window: their cells are
    // the window's less the mirror's offset.
    for mirror in &visit.mirrors {
        let SiteId::Poi(chunk) = mirror.id else {
            continue;
        };
        // Only towns have shops (and this leaves other sites' saves alone).
        let has_town = super::wild_chunk(world, chunk);
        if !has_town.is_some_and(|wild| !wild.town.is_empty()) {
            continue;
        }
        let Some(building) = keeps(world, mirror.id, mirror.index, with) else {
            continue;
        };
        let town = &world.wilderness.chunk(chunk)?.town;
        let found = town.iter().find(|b| b.look.id == building)?;
        let (door_x, door_y) = found.look.door;
        let (ox, oy) = mirror.offset;
        return Some(ShopHere {
            id: mirror.id,
            index: mirror.index,
            poi: world.pois.in_chunk(chunk)?,
            building: found.clone(),
            of_kind: of_kind(town, found),
            door: (door_x - ox, door_y - oy),
        });
    }
    None
}

/// Which building of a site's level `with` keeps, if any.
fn keeps(world: &mut ServerWorld, id: SiteId, index: u32, with: Id) -> Option<u16> {
    let level = world.site(id).levels.get(&index)?;
    let mut buildings = level.buildings.iter();
    buildings
        .find(|(_, save)| save.keeper == Some(with))
        .map(|(&building, _)| building)
}

/// How many of a town's buildings are of the same kind as `building`.
fn of_kind(town: &[TownBuilding], building: &TownBuilding) -> u32 {
    let same = town.iter().filter(|b| b.look.kind == building.look.kind);
    same.count() as u32
}
