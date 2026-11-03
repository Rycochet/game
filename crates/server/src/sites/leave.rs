//! Leaving sites: forgetting them, being put out, using the exit.

use shared::poi::Poi;
use shared::protocol::ServerMessage;
use shared::site::{Exit, leads_out};

use super::{Arrival, NewVisit, Visit, enter_level, enter_site};
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
    let Visit {
        poi,
        id,
        index,
        position,
        ..
    } = visit;
    world.forget_site(id);
    client.character.memory.remove(&id);
    client.character.bags.remove(&id);
    client.character.site_versions.remove(&id);
    enter_site(client, world, poi, id, index, Arrival::At(position))
}

/// Puts the player out of the site they are on, which is no longer what they
/// are looking at, at its poi. They can go in again.
pub(crate) fn put_out(client: &mut Client, world: &mut ServerWorld) -> bool {
    let Some(visit) = client.visit.take() else {
        return true;
    };
    leave_for_poi(client, world, visit.poi, visit.id)
}

/// The character forgets a site that has changed, and is on the world map at
/// its poi.
pub(super) fn leave_for_poi(
    client: &mut Client,
    world: &mut ServerWorld,
    poi: Poi,
    id: SiteId,
) -> bool {
    client.character.memory.remove(&id);
    client.character.bags.remove(&id);
    client.character.site_versions.remove(&id);
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
    let exit = match stairs {
        Some(stairs) => stairs.to,
        None if leads_out(&visit.level.level, x, y) => Exit::WorldMap,
        None => return true,
    };
    leave(client, world);
    let visit = client.visit.take().expect("checked above");
    client.unsaved = true;
    match exit {
        Exit::Level(index) => {
            // A building's stairs are on the same cell of both floors, and it
            // may have several: arrive at the ones taken.
            let from = if visit.site.is_building() {
                Arrival::At((x, y))
            } else {
                Arrival::From(Exit::Level(visit.index))
            };
            let (visit_id, from_level) = (visit.id, visit.index);
            let new = NewVisit {
                poi: visit.poi,
                id: visit.id,
                site: visit.site,
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
            let (x, y) = (visit.poi.x, visit.poi.y);
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
        let site = world.site_mut(visit.id, visit.poi.kind);
        site.levels.entry(visit.index).or_default().last_visit = now();
    }
}
