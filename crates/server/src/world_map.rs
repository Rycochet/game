//! Sending each player the world map chunks coming into their sight, and
//! remembering the pois they have seen.

use std::collections::btree_map::Entry;

use bevy::prelude::*;
use shared::movement::{Step, step_on_world_map};
use shared::poi::{ChunkPos, Poi};
use shared::protocol::{ServerMessage, WORLD_MAP_PRELOAD, WORLD_MAP_SIGHT, WorldChunk};
use shared::showcase;

use crate::connections::Client;
use crate::storage::KnownPoi;
use crate::{ServerWorld, sites};

/// The player takes a step on the world map, unless it leads into water (a
/// refused step is left for `Position` to put right). `false` if the client
/// has gone.
pub(crate) fn stepped(client: &mut Client, world: &ServerWorld, step: Step) -> bool {
    let conditions = world.conditions();
    let Some(to) = step_on_world_map(&world.ground, conditions, client.world_position(), step)
    else {
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

/// Where new characters start on the world map (`CharacterWorldSave`'s default).
pub(crate) const START: (i64, i64) = (0, 0);

/// Developer tools: puts the player at `(x, y)` on the world map.
pub(crate) fn teleport(client: &mut Client, world: &ServerWorld, x: i64, y: i64) -> bool {
    client.character.world_position = (x, y);
    client.unsaved = true;
    client.changed_map();
    client.link.send(&ServerMessage::Teleported { x, y }) && look_around(client, world)
}

/// Developer tools: puts the player back where new characters start, out of
/// any site.
pub(crate) fn back_to_start(client: &mut Client, world: &mut ServerWorld) -> bool {
    go_to(client, world, START)
}

/// Developer tools: puts the player on the plain with every kind of poi, out of
/// any site.
pub(crate) fn to_showcase(client: &mut Client, world: &mut ServerWorld) -> bool {
    !showcase::ENABLED || go_to(client, world, showcase::ARRIVAL)
}

/// Puts the player somewhere on the world map, out of any site.
fn go_to(client: &mut Client, world: &mut ServerWorld, (x, y): (i64, i64)) -> bool {
    sites::leave(client, world);
    client.visit = None;
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
    let seen = client.character.pois.get(&chunk);
    world.pois.in_chunk(chunk).filter(|poi| {
        poi.kind.is_town()
            || client.reveal_pois
            || seen.is_some_and(|seen| seen.known || seen.visited)
    })
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
    client.character.memory.clear();
    client.character.bags.clear();
    client.character.site_versions.clear();
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
            let poi = known_poi(client, world, chunk).map(Into::into);
            if !client
                .link
                .send(&ServerMessage::WorldChunk(WorldChunk { chunk, poi }))
            {
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
    let chunks = world
        .pois
        .chunks_in_area((x - reach, y - reach), (x + reach, y + reach));
    for chunk in chunks {
        if client.world_chunks.insert(chunk) {
            let poi = known_poi(client, world, chunk).map(Into::into);
            if !client
                .link
                .send(&ServerMessage::WorldChunk(WorldChunk { chunk, poi }))
            {
                return false;
            }
        }
    }
    true
}

#[cfg(test)]
mod tests;
