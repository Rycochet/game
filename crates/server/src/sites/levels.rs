//! Making and entering a level, and telling the client about its other floors.

use bevy::prelude::*;
use shared::dungeon::Level;
use shared::dungeon::Sight;
use shared::poi::Poi;
use shared::protocol::{
    LevelInfo, LevelKind, RememberedLevel, SeenStairs, ServerMessage, level_chunk,
};
use shared::shop::sells;
use shared::site::{Exit, Site, SiteLevel, Stairs, StairsDirection};

use super::remembered::{mirror_of_first, send_remembered};
use super::{
    Arrival, DoorSite, NewKeepers, Place, Visit, VisitDoors, beside, door_looks, ensure_locks,
    keepers_of, level_chunks, place_name_of, set_doors, update_sight,
};
use crate::ServerWorld;
use crate::connections::Client;
use crate::storage::{CellMasks, SiteId, now};

/// The site a player is going to a level of.
pub(super) struct NewVisit {
    pub(super) poi: Poi,
    pub(super) id: SiteId,
    pub(super) site: Site,
    /// They came in from the wilderness, where leaving takes them back.
    pub(super) from_wilderness: bool,
}

/// Level `index` of a site with players' changes over it, its stairs as
/// players see them, and its doors as they are now.
pub(super) fn level_of(
    world: &mut ServerWorld,
    new: &NewVisit,
    index: u32,
) -> (SiteLevel, Vec<SeenStairs>, VisitDoors) {
    let site = &new.site;
    let mut level = site.level(index);
    let saved = world.site_mut(new.id).levels.entry(index).or_default();
    for (&(x, y), &cell) in &saved.changes {
        level.level.set(x, y, cell);
    }
    let stairs = level
        .stairs
        .iter()
        .map(|&stairs| seen_stairs(site, index, &level.level, stairs))
        .collect();
    ensure_locks(world, (new.id, &new.poi), index, &level);
    let door_site = DoorSite {
        id: new.id,
        index,
        poi: &new.poi,
    };
    let bounds = level.level.bounds();
    let doors = set_doors(world, &door_site, &mut level.level, bounds, (0, 0));
    (level, stairs, doors)
}

/// Stairs on level `index` of a site (`level`) as players see them.
pub(super) fn seen_stairs(site: &Site, index: u32, level: &Level, stairs: Stairs) -> SeenStairs {
    SeenStairs {
        x: stairs.x,
        y: stairs.y,
        direction: stairs.direction,
        // Only for looks: the other level is generated without changes.
        descends: site.stairs_descend(index, level, stairs),
        style: site.stairs_style(index, stairs),
        open_to_sky: match stairs.to {
            Exit::WorldMap => true,
            // Also only for looks, and only worth generating going up.
            Exit::Level(other) => {
                stairs.direction == StairsDirection::Up
                    && site
                        .level(other)
                        .level
                        .cell(stairs.x, stairs.y)
                        .is_outside()
            }
        },
    }
}

/// Sends what the character remembers of the site's levels other than
/// `index`, for the maps.
pub(super) fn send_other_levels(
    client: &Client,
    world: &mut ServerWorld,
    new: &NewVisit,
    index: u32,
) -> bool {
    let Some(memory) = client.character.memory.get(&new.id) else {
        return true;
    };
    for (&other, remembered) in memory {
        // (A level may be gone, if the generator has changed.)
        let known = remembered.values().any(|&mask| mask != 0);
        if other == index || other >= new.site.levels() || !known {
            continue;
        }
        let (level, stairs, doors) = level_of(world, new, other);
        let level = &level.level;
        let looks = |x, y| {
            door_looks(
                client,
                &doors,
                (new.id, other),
                level.cell(x, y),
                (x, y),
                (x, y),
                true,
            )
        };
        let is_remembered = |x, y| {
            let (chunk, bit) = level_chunk(x, y);
            remembered.get(&chunk).is_some_and(|m| m & (1 << bit) != 0)
        };
        let neighbours = beside(level, is_remembered, remembered);
        let truesight = client.truesight;
        let mut chunks = level_chunks(level, &stairs, remembered, true, truesight, &looks);
        chunks.extend(level_chunks(
            level,
            &stairs,
            &neighbours,
            false,
            truesight,
            &looks,
        ));
        let message = ServerMessage::RememberedLevel(RememberedLevel {
            index: other,
            bounds: level.bounds(),
            chunks,
        });
        if !client.link.send(&message) {
            return false;
        }
    }
    true
}

/// Puts the player on level `index` of a site: generates it, tells the client
/// about it, sends what the player remembers of it, then what they see. On
/// coming into the site, also what they remember of its other levels.
pub(super) fn enter_level(
    client: &mut Client,
    world: &mut ServerWorld,
    new: NewVisit,
    index: u32,
    arrival: Arrival,
    entering_site: bool,
) -> bool {
    let (level, stairs, doors) = level_of(world, &new, index);
    let saved = world.site_mut(new.id).levels.entry(index);
    saved.or_default().last_visit = now();
    let site = &new.site;
    let position = match arrival {
        Arrival::At((x, y)) if level.level.is_walkable(x, y) => (x, y),
        Arrival::At(_) => level.arrive(Exit::WorldMap),
        Arrival::From(from) => level.arrive(from),
    };
    world.move_mob_aside((new.id, index), position);
    let info = LevelInfo {
        kind: LevelKind::Poi(site.kind()),
        ruined: site.is_ruined(),
        building: site.is_building(),
        index,
        levels: site.levels(),
        basement: site.has_basement(),
        bounds: level.level.bounds(),
        arrival: position,
        entering_site,
        place_name: place_name_of(world, &new.poi),
    };
    let view = Sight::new(&level.level);
    world.populate(new.id, new.poi, index, &level);
    let looked_from = Sight::new(&level.level);
    let loaded = Sight::new(&level.level);
    client.changed_map();
    if !client.link.send(&ServerMessage::EnterLevel(info)) {
        return false;
    }
    if entering_site && !send_other_levels(client, world, &new, index) {
        return false;
    }
    let mirrors = mirror_of_first(world, &new, index, &level);
    let NewVisit {
        poi,
        id,
        site,
        from_wilderness,
    } = new;
    let mut visit = Visit {
        looked_from,
        loaded,
        view,
        view_from: None,
        place: Place::Site {
            poi,
            site: Box::new(site),
            from_wilderness,
        },
        id,
        index,
        level,
        stairs,
        doors,
        position,
        sent: CellMasks::new(),
        mirrors,
        wild_area: 0,
    };
    if visit.is_town() {
        let looks = visit.level.buildings.iter().map(|b| b.look.clone());
        let message = ServerMessage::Buildings(looks.collect());
        if !client.link.send(&message) {
            return false;
        }
        let shops = visit.level.buildings.iter();
        let shops = shops.filter(|b| !sells(b.look.kind).is_empty());
        let keepers = NewKeepers {
            id: visit.id,
            index,
            shops: shops.map(|b| b.look.id).collect(),
        };
        let message = ServerMessage::Keepers(keepers_of(world, &keepers));
        if !client.link.send(&message) {
            return false;
        }
    }
    if !visit.level.fields.is_empty() {
        let message = ServerMessage::Fields(visit.level.fields.clone());
        if !client.link.send(&message) {
            return false;
        }
    }
    if !send_remembered(client, &mut visit) {
        return false;
    }
    client.visit = Some(visit);
    client.character.hall = None;
    client.unsaved = true;
    update_sight(client)
}

/// Sends the level the player is on again, as if arriving where they stand.
pub(super) fn arrive_again(client: &mut Client, world: &mut ServerWorld) -> bool {
    let Some(visit) = client.visit.take() else {
        return true;
    };
    let from_wilderness = visit.came_from_wilderness();
    let Visit {
        place,
        id,
        index,
        position,
        ..
    } = visit;
    let Place::Site { poi, site, .. } = place else {
        return super::wild::enter_wilderness(client, world, position);
    };
    let new = NewVisit {
        poi,
        id,
        site: *site,
        from_wilderness,
    };
    enter_level(client, world, new, index, Arrival::At(position), false)
}
