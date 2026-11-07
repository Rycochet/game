//! Leaving sites: forgetting them, being put out, using the exit.

use shared::poi::Poi;
use shared::protocol::ServerMessage;
use shared::site::{Exit, leads_out};

use super::wild::{back_to_wilderness, enter_wilderness, into_site};
use super::{Arrival, NewVisit, Place, Visit, enter_level, enter_site};
use crate::ServerWorld;
use crate::connections::Client;
use crate::storage::{SiteId, now};
use crate::world_map::look_around;

/// Developer tools: the server forgets the site the player is on, and they
/// arrive on it again as first made, on the same level and cell if they can.
pub(crate) fn forget_site(client: &mut Client, world: &mut ServerWorld) -> bool {
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
    world.forget_site(id);
    client.character.forget_site(&id);
    let Place::Site { poi, .. } = place else {
        return enter_wilderness(client, world, position);
    };
    let at = Arrival::At(position);
    enter_site(client, world, poi, id, index, at, from_wilderness)
}

/// Puts the player out of the site they are on, which is no longer what they
/// are looking at, at its poi. They can go in again.
pub(crate) fn put_out(client: &mut Client, world: &mut ServerWorld) -> bool {
    let Some(visit) = client.visit.take() else {
        return true;
    };
    match visit.poi() {
        Some(&poi) => leave_for_poi(client, world, poi, visit.id),
        None => {
            // The wilderness: onto the world map where they are.
            let (x, y) = client.world_position();
            client.changed_map();
            client.link.send(&ServerMessage::OnWorldMap { x, y }) && look_around(client, world)
        }
    }
}

/// The character forgets a site that has changed, and is on the world map at
/// its poi.
pub(super) fn leave_for_poi(
    client: &mut Client,
    world: &mut ServerWorld,
    poi: Poi,
    id: SiteId,
) -> bool {
    client.character.forget_site(&id);
    client.character.world_position = (poi.x, poi.y);
    client.unsaved = true;
    client.changed_map();
    let (x, y) = (poi.x, poi.y);
    client.link.send(&ServerMessage::OnWorldMap { x, y }) && look_around(client, world)
}

/// Uses what the player stands on: stairs to another level, or ground on
/// the edge of the level back to the world map.
pub(crate) fn use_exit(client: &mut Client, world: &mut ServerWorld) -> bool {
    let Some(visit) = &client.visit else {
        return true;
    };
    let (x, y) = visit.position;
    let stairs = visit.level.stairs.iter().find(|s| (s.x, s.y) == (x, y));
    if visit.is_wilderness() {
        return match stairs {
            Some(stairs) => into_site(client, world, (x, y), stairs.to),
            None => true,
        };
    }
    let exit = match stairs {
        Some(stairs) => stairs.to,
        None if leads_out(&visit.level.level, x, y) => Exit::WorldMap,
        None => return true,
    };
    if let SiteId::DevArea(_) = visit.id {
        return super::dev_area::back_from_area(client, world);
    }
    if visit.came_from_wilderness()
        && let Some(at) = back_to_wilderness(world, visit, exit, (x, y))
    {
        leave(client, world);
        client.visit = None;
        return enter_wilderness(client, world, at);
    }
    leave(client, world);
    let visit = client.visit.take().expect("checked above");
    client.unsaved = true;
    let from_wilderness = visit.came_from_wilderness();
    let (visit_id, from_level) = (visit.id, visit.index);
    let Place::Site { poi, site, .. } = visit.place else {
        return true;
    };
    match exit {
        Exit::Level(index) => {
            // A building's stairs are on the same cell of both floors, and it
            // may have several: arrive at the ones taken.
            let from = if site.is_building() {
                Arrival::At((x, y))
            } else {
                Arrival::From(Exit::Level(from_level))
            };
            let new = NewVisit {
                poi,
                id: visit_id,
                site: *site,
                from_wilderness,
            };
            let arrived = enter_level(client, world, new, index, from, false);
            if let Some(there) = client.visit.as_ref().map(Visit::position) {
                let site = visit_id;
                let taken = crate::entities::StairsTaken {
                    who: client.entity_id(),
                    site,
                    from_level,
                    stairs: (x, y),
                    to_level: index,
                    arrives: there,
                };
                if !client.god_mode {
                    world.saw_stairs_taken(taken);
                }
            }
            arrived
        }
        Exit::WorldMap => {
            // Coming back in, they arrive where they walked out.
            let left_at = (from_level, (x, y));
            client.character.left_at.insert(visit_id, left_at);
            let (x, y) = (poi.x, poi.y);
            client.character.world_position = (x, y);
            client.changed_map();
            client.link.send(&ServerMessage::OnWorldMap { x, y }) && look_around(client, world)
        }
    }
}

/// The player is leaving their level (or the server): it was last visited
/// now. The visit itself is left alone (saved with the character).
pub(crate) fn leave(client: &Client, world: &mut ServerWorld) {
    if let Some(visit) = &client.visit {
        let site = world.site_mut(visit.id);
        site.levels.entry(visit.index).or_default().last_visit = now();
    }
}

/// Killed: out of the site (or the wilderness) onto the world map, at its poi or
/// where they were, keeping what they know of it. `false` if the client has gone.
pub(crate) fn killed(client: &mut Client, world: &mut ServerWorld) -> bool {
    if client
        .visit
        .as_ref()
        .is_some_and(|visit| matches!(visit.id, SiteId::DevArea(_)))
    {
        return super::dev_area::back_from_area(client, world);
    }
    leave(client, world);
    // Worked out while the visit says where in the wilderness they are.
    let here = client.world_position();
    let Some(visit) = client.visit.take() else {
        return true;
    };
    let (x, y) = visit.poi().map_or(here, |poi| (poi.x, poi.y));
    client.character.world_position = (x, y);
    client.unsaved = true;
    client.changed_map();
    client.link.send(&ServerMessage::OnWorldMap { x, y }) && look_around(client, world)
}
