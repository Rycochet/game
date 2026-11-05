//! Moving a player from the world they are in to another, while playing: by
//! dev tools, or through a Hall gate onto its planet.

use bevy::prelude::*;
use shared::protocol::ServerMessage;
use shared::world::WorldSeed;

use super::Client;
use super::welcome::send_welcome;
use crate::{ServerOptions, ServerWorlds, sites, world_map};

/// Where on a world's map someone moved to it arrives.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Arrival {
    /// Where they were, or at one of its World Gates if they are new there
    /// (dev tools).
    AsBefore,
    /// At this World Gate of it (through a Hall gate).
    Gate(u8),
}

impl Client {
    /// Has the player moved to the world of `seed` once its messages this tick
    /// have been read (`handle_messages`), as that needs every world.
    pub(crate) fn go_to_world(&mut self, seed: WorldSeed, arrival: Arrival) {
        self.going_to = Some((seed, arrival));
    }
}

/// Moves a client to the world of `seed`, opening it if need be: out of any
/// site or the Hall of the world they leave (saved, with what they know of it
/// kept for coming back), onto the new one's world map as `arrival` says. Told
/// by a fresh `Welcome`, then `OnWorldMap`. To the world they are in, only a
/// gate moves them (out of the Hall, onto its map). A world that can't be
/// opened leaves them where they are. `false` if the client has gone.
pub(super) fn change_world(
    client: &mut Client,
    worlds: &mut ServerWorlds,
    (seed, arrival): (WorldSeed, Arrival),
    options: &ServerOptions,
) -> bool {
    let id = client.character_id.expect("only players move");
    let to = match worlds.open(seed) {
        Ok(to) => to,
        Err(error) => {
            warn!("Can't move {id:?} to world {:?}: {error}", seed.id().0);
            return true;
        }
    };
    let world = worlds.of(client.world);
    if world.seed.id() == to {
        return match arrival.gate_in(world) {
            Some(gate) => world_map::go_to(client, world, gate),
            None => true,
        };
    }
    sites::leave(client, world);
    client.visit = None;
    // Coming back, they are on its world map, not in the Hall.
    client.character.hall = None;
    client.unsaved = true;
    client.save(world);
    // Read back with what was just saved (the queue answers first), so what
    // they know of the world they left goes in with the others.
    let world = worlds.of(Some(to));
    match world.storage.load_character(&id, to) {
        Ok((here, others)) => (client.character, client.other_worlds) = (here, others),
        Err(error) => return client.refuse_for_saves(&error),
    }
    info!("{id:?} moved to world {:?}", to.0);
    client.world = Some(to);
    client.character.visit = None;
    client.character.hall = None;
    if let Some(gate) = arrival.gate_in(world) {
        client.character.world_position = gate;
    } else if client.character.forgotten_to == 0 {
        client.character.world_position = world_map::arrival(world, &id);
    }
    client.world_chunks.clear();
    client.sent_roads.clear();
    // Saved as in the new world straight away, so a crash can't split them.
    client.unsaved = true;
    client.save(world);
    client.changed_map();
    let (x, y) = client.world_position();
    send_welcome(client, world, options)
        && client.link.send(&ServerMessage::OnWorldMap { x, y })
        && world_map::look_around(client, world)
}

impl Arrival {
    /// The World Gate of `world` it arrives at, if it is one it has.
    fn gate_in(self, world: &crate::ServerWorld) -> Option<(i64, i64)> {
        match self {
            Self::AsBefore => None,
            Self::Gate(entrance) => world.gate_position(entrance),
        }
    }
}
