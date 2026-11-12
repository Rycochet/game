//! Doors on a visit's level (`shared::door`): each one's state worked out
//! when the level is made (as rolled, or as someone left it: `LevelSave::doors`)
//! and set on the level for the rules; opening and closing them; and telling
//! everyone else on the level when one changes. A door in a poi's cutout in the
//! wilderness is its site's (`Mirror`), so both see the same.

use shared::door::{Door, Side, Swing, door_is_outside, starting_door};
use shared::dungeon::{Cell, Level};
use shared::poi::Poi;
use shared::site::PoiLook;
use std::collections::BTreeMap;

use shared::protocol::DoorAsk;

use crate::ServerWorld;
use crate::connections::Client;
use crate::entities::Opened;
use crate::sites::Visit;
use crate::storage::SiteId;

// Split into files (`cargo xtask lint` keeps files small).
mod act;
mod kick;
mod locks;
mod looks;
mod share;

pub(super) use act::changed_at;
use act::{close_near, dev_all, dev_key, open_door};
pub(crate) use kick::{door_to_kick, kick_door};
pub(super) use locks::ensure_locks;
pub(super) use looks::{door_looks, remember_doors};
pub(crate) use share::{DoorChange, Standing, share_doors};

/// Where a door is kept: the site, level and cell it belongs to.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) struct DoorKey {
    pub(crate) site: SiteId,
    pub(crate) index: u32,
    pub(crate) at: (i64, i64),
}

/// A door on a visit's level: where it is kept, and how it is now.
#[derive(Clone, Copy, Debug)]
pub(crate) struct VisitDoor {
    pub(crate) key: DoorKey,
    pub(crate) door: Door,
}

/// A visit's doors, by their cells on its level.
pub(crate) type VisitDoors = BTreeMap<(i64, i64), VisitDoor>;

/// The site a level's doors belong to, to work them out.
pub(super) struct DoorSite<'a> {
    pub(super) id: SiteId,
    pub(super) index: u32,
    pub(super) poi: &'a Poi,
}

/// The doors in `bounds` of `level` (`(min_x, min_y, width, height)`), whose
/// cells less `offset` are the site's own, as they are now: rolled, or as
/// someone left them. Sets their swings on the level.
pub(super) fn set_doors(
    world: &mut ServerWorld,
    site: &DoorSite,
    level: &mut Level,
    (min_x, min_y, width, height): (i64, i64, i64, i64),
    offset: (i64, i64),
) -> VisitDoors {
    let saved = world.site(site.id).levels.get(&site.index);
    let saved = saved.map(|level| level.doors.clone()).unwrap_or_default();
    let poi = site.poi;
    let ruined = PoiLook::of(poi.kind, poi.seed, poi.desert).is_ruined();
    let mut doors = VisitDoors::new();
    for y in min_y..min_y + height {
        for x in min_x..min_x + width {
            let cell = level.cell(x, y);
            if !matches!(cell, Cell::Door | Cell::SecretDoor) {
                continue;
            }
            let at = (x - offset.0, y - offset.1);
            let door = saved
                .get(&at)
                .copied()
                .unwrap_or_else(|| first_door(poi, ruined, site.index, level, (x, y), at));
            level.set_swing(x, y, door.swing);
            level.set_locked(x, y, door.is_locked());
            let key = DoorKey {
                site: site.id,
                index: site.index,
                at,
            };
            doors.insert((x, y), VisitDoor { key, door });
        }
    }
    doors
}

/// How the door at `here` on `level` (the site's cell `at`) started out.
fn first_door(
    poi: &Poi,
    ruined: bool,
    index: u32,
    level: &Level,
    (x, y): (i64, i64),
    at: (i64, i64),
) -> Door {
    let outside = door_is_outside(level, x, y);
    let runs = level.opening_runs_east_west(x, y);
    let secret = level.cell(x, y) == Cell::SecretDoor;
    starting_door(poi, ruined, index, at, secret, outside, runs)
}

/// A mob has opened a door on its site level: kept, and told to everyone there.
pub(crate) fn mob_opened(world: &mut ServerWorld, opened: Opened) {
    let (site, index) = opened.place;
    let key = DoorKey {
        site,
        index,
        at: opened.at,
    };
    let saved = world.site(site).levels.get(&index);
    let saved = saved.and_then(|level| level.doors.get(&opened.at).copied());
    let door = saved.or_else(|| {
        let level = world.populations.0.get(&opened.place)?.level();
        let poi = &opened.poi;
        let ruined = PoiLook::of(poi.kind, poi.seed, poi.desert).is_ruined();
        Some(first_door(poi, ruined, index, level, opened.at, opened.at))
    });
    if let Some(mut door) = door {
        door.swing = opened.swing;
        keep_door(world, key, door);
    }
}

/// Developer tools: the visit's cell `(x, y)`, kept as `key`, has just been
/// changed: a new door there starts open (away from the player), a new secret
/// door closed, and one no longer there is forgotten.
pub(super) fn cell_changed(
    world: &mut ServerWorld,
    visit: &mut Visit,
    (x, y): (i64, i64),
    key: DoorKey,
) {
    if let Some(saved) = world.site_mut(key.site).levels.get_mut(&key.index) {
        saved.doors.remove(&key.at);
    }
    let level = &visit.level.level;
    let cell = level.cell(x, y);
    if !matches!(cell, Cell::Door | Cell::SecretDoor) {
        visit.doors.remove(&(x, y));
        return;
    }
    let poi = visit.poi().copied().or_else(|| {
        let SiteId::Poi(chunk) = key.site else {
            return None;
        };
        super::wild_chunk(world, chunk)?
            .plan
            .cutout()
            .map(|cutout| *cutout.poi())
    });
    let Some(poi) = poi else {
        return;
    };
    let mut door = first_door(&poi, false, key.index, level, (x, y), key.at);
    door.swing = if cell == Cell::Door {
        let runs = level.opening_runs_east_west(x, y);
        Swing::Open(Side::of(runs, (x, y), visit.position()).opposite())
    } else {
        Swing::Closed
    };
    visit.level.level.set_swing(x, y, door.swing);
    visit.doors.insert((x, y), VisitDoor { key, door });
}

/// Does what a client asked of doors (`DoorAsk`), in turn with its steps:
/// whether it is still there, and what it cost.
pub(crate) fn door_act(client: &mut Client, world: &mut ServerWorld, ask: DoorAsk) -> (bool, u32) {
    match ask {
        DoorAsk::Open { x, y } => open_door(client, world, x, y),
        DoorAsk::CloseNear => close_near(client, world),
        DoorAsk::DevAll { open } => (dev_all(client, world, open), 0),
        DoorAsk::DevKey => (dev_key(client, world), 0),
    }
}

impl Visit {
    /// The doors of the visit's level, for tests.
    #[cfg(test)]
    pub(crate) fn doors_for_test(&self) -> &VisitDoors {
        &self.doors
    }
}

/// Keeps a door as it is now, for everyone: in its level's save, and to be
/// told to everyone else on it (`share_doors`).
pub(super) fn keep_door(world: &mut ServerWorld, key: DoorKey, door: Door) {
    let saved = world
        .site_mut(key.site)
        .levels
        .entry(key.index)
        .or_default();
    saved.doors.insert(key.at, door);
    world.door_changes.push(DoorChange { key, door });
}

#[cfg(test)]
mod tests;
