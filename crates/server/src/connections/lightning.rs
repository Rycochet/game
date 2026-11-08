//! Telling players when lightning flashes where they are: looked for once a
//! second of the server's clock, so not while it is paused.

use bevy::prelude::*;
use shared::protocol::ServerMessage;

use super::systems::for_each_world;
use super::{Client, Connections, Phase};
use crate::{ServerWorld, ServerWorlds};

/// Every tick: on each new whole second of the server's clock, tells everyone
/// playing on a world (on its map, in its wilderness or in a site, by where
/// they are on its map) if lightning flashes there. Nobody in the Hall of
/// Worlds, which has no sky.
pub(crate) fn tell_of_lightning(
    mut connections: ResMut<Connections>,
    mut worlds: ResMut<ServerWorlds>,
) {
    for_each_world(&mut connections.clients, &mut worlds, |clients, world| {
        let second = world.server_clock.seconds().floor() as i64;
        if second == world.lightning_second {
            return;
        }
        world.lightning_second = second;
        tell(clients, world, second);
    });
}

/// Tells everyone under the sky whom lightning flashes for in this second.
pub(super) fn tell(clients: &mut [Client], world: &ServerWorld, second: i64) {
    for client in clients.iter_mut().filter(|client| under_sky(client)) {
        let (x, y) = client.world_position();
        if world.lightning_at(x, y, second) {
            client.link.send(&ServerMessage::Lightning);
        }
    }
}

/// Whether a client is playing somewhere with weather.
fn under_sky(client: &Client) -> bool {
    let in_hall = client.visit.is_none() && client.character.hall.is_some();
    client.phase == Phase::Playing && !in_hall
}

#[cfg(test)]
mod tests;
