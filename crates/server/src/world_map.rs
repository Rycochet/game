//! Sending each player the world map chunks coming into their sight, and
//! remembering the pois they have seen.

mod arrival;
mod place_names;

use std::collections::btree_map::Entry;

use bevy::prelude::*;
use shared::movement::{Gait, Step, step_on_world_map_by};
use shared::poi::{ChunkPos, Poi};
use shared::protocol::{ServerMessage, WORLD_MAP_PRELOAD, WORLD_MAP_SIGHT};

use crate::connections::Client;
use crate::storage::KnownPoi;
use crate::{ServerWorld, sites};
pub(crate) use arrival::{arrival, nearest_gate};
use place_names::chunk_message;
pub(crate) use place_names::{learn_place, learn_place_name, send_chunk};

/// The player takes a step on the world map, unless it leads into water (but
/// a good swimmer may swim the sea near the coast) (a
/// refused step is left for `Position` to put right). `false` if the client
/// has gone.
pub(crate) fn stepped(client: &mut Client, world: &ServerWorld, step: Step) -> bool {
    let conditions = world.conditions();
    let gait = Gait::of(&client.sheet.skills);
    let from = client.world_position();
    let Some(to) = step_on_world_map_by(&world.ground, conditions, from, step, gait) else {
        info!(
            "Refused a step into water from {:?}",
            client.world_position()
        );
        return true;
    };
    client.character.world_position = to;
    client.unsaved = true;
    look_around(client, world)
}

/// Developer tools: puts the player at `(x, y)` on the world map.
pub(crate) fn teleport(client: &mut Client, world: &ServerWorld, x: i64, y: i64) -> bool {
    client.character.world_position = (x, y);
    client.unsaved = true;
    client.changed_map();
    client.link.send(&ServerMessage::Teleported { x, y }) && look_around(client, world)
}

/// Developer tools: puts the player at the World Gate nearest them, where
/// characters arrive, out of any site.
pub(crate) fn back_to_start(client: &mut Client, world: &mut ServerWorld) -> bool {
    let gate = nearest_gate(world, client.world_position());
    go_to(client, world, gate)
}

/// Puts the player somewhere on the world map, out of any site or the Hall.
pub(crate) fn go_to(client: &mut Client, world: &mut ServerWorld, (x, y): (i64, i64)) -> bool {
    sites::leave(client, world);
    client.visit = None;
    client.character.hall = None;
    client.character.world_position = (x, y);
    client.unsaved = true;
    client.changed_map();
    client.link.send(&ServerMessage::OnWorldMap { x, y }) && look_around(client, world)
}

/// TODO: for now every character knows of every poi (the owner's choice). Turn
/// this off once characters have means of learning of them: see `decisions.md`.
pub(crate) const ALL_POIS_KNOWN: bool = true;

/// The poi of a chunk, if this client may know of it: towns are seen by all,
/// anything else only once the character has learned of it.
pub(crate) fn known_poi(client: &Client, world: &ServerWorld, chunk: ChunkPos) -> Option<Poi> {
    let poi = world.pois.in_chunk(chunk)?;
    knows_of(client, chunk, poi.kind).then_some(poi)
}

/// Whether this client may know of the poi of `kind` in `chunk` (see
/// `known_poi`).
pub(crate) fn knows_of(client: &Client, chunk: ChunkPos, kind: shared::poi::PoiKind) -> bool {
    let seen = client.character.pois.get(&chunk);
    kind.is_town() || client.reveal_pois || seen.is_some_and(|seen| seen.known || seen.visited)
}

/// Developer tools: shows every poi, on this connection only.
pub(crate) fn reveal_pois(client: &mut Client, world: &ServerWorld) -> bool {
    client.reveal_pois = true;
    client.world_chunks.clear();
    look_around(client, world)
}

/// Developer tools: the character forgets the whole world map and every site
/// on it, and is told again what is in sight.
pub(crate) fn forget(client: &mut Client, world: &ServerWorld) -> bool {
    client.character.pois.clear();
    client.character.forget_sites();
    client.world_chunks.clear();
    client.unsaved = true;
    client.link.send(&ServerMessage::ForgetWorldMap) && look_around(client, world)
}

/// Developer tools: as `forget`, and the server forgets every site, to be made
/// anew when next entered.
pub(crate) fn forget_on_server(client: &mut Client, world: &mut ServerWorld) -> bool {
    world.forget_sites();
    forget(client, world)
}

/// The player is somewhere new on the world map: the chunks now in sight are
/// known from now on (their land as well as any poi, for the player's
/// map), a new connection is sent all they know, and the chunks within sight, plus a margin, that
/// the client hasn't been sent yet are sent. `false` if the client has
/// gone.
pub(crate) fn look_around(client: &mut Client, world: &ServerWorld) -> bool {
    if client.world_chunks.is_empty() {
        let known: Vec<_> = client.character.pois.keys().copied().collect();
        for chunk in known {
            client.world_chunks.insert(chunk);
            if !client.link.send(&chunk_message(client, world, chunk)) {
                return false;
            }
        }
    }
    let (x, y) = client.world_position();
    let sight = WORLD_MAP_SIGHT;
    for chunk in world
        .pois
        .chunks_in_area((x - sight, y - sight), (x + sight, y + sight))
    {
        let known = &mut client.character.pois;
        if let Entry::Vacant(entry) = known.entry(chunk) {
            entry.insert(KnownPoi::default());
            client.unsaved = true;
        }
    }
    let reach = WORLD_MAP_SIGHT + WORLD_MAP_PRELOAD;
    let (min, max) = ((x - reach, y - reach), (x + reach, y + reach));
    let mut new = false;
    for chunk in world.pois.chunks_in_area(min, max) {
        if client.world_chunks.insert(chunk) {
            new = true;
            if !client.link.send(&chunk_message(client, world, chunk)) {
                return false;
            }
        }
    }
    // The roads through them, sent by `connections::send_roads` once found.
    if new {
        world.ask_roads(min, max);
    }
    true
}

#[cfg(test)]
mod tests;
