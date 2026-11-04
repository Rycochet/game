//! Telling players of the roads near them, as the road thread finds them.

use bevy::prelude::*;
use shared::protocol::{ServerMessage, WORLD_MAP_PRELOAD, WORLD_MAP_SIGHT};
use shared::road::Road;

use super::{Connections, Phase};
use crate::ServerWorlds;

/// Sends each road the road thread has found to every player in its world near
/// enough to be sent any of it (as far as world map chunks are sent), once per
/// connection, and from now on steps along it cost less. Roads of a world that
/// has closed meanwhile are dropped: it asks again on opening.
pub(crate) fn send_roads(mut connections: ResMut<Connections>, mut worlds: ResMut<ServerWorlds>) {
    let found = worlds.home().roads_found();
    for (id, road) in &found {
        let Some(world) = worlds.get_mut(*id) else {
            continue;
        };
        world.road_steps.add(road);
        let playing = |client: &&mut super::Client| {
            !client.gone && client.phase == Phase::Playing && client.world == Some(*id)
        };
        for client in connections.clients.iter_mut().filter(playing) {
            if client.sent_roads.contains(&road.link)
                || !near(road, client.world_position(), world.width)
            {
                continue;
            }
            // A client that has gone is dealt with when its messages are read.
            if client.link.send(&ServerMessage::Road(Road::clone(road))) {
                client.sent_roads.insert(road.link);
            }
        }
    }
}

/// Whether any of a road is within reach of a player at `(x, y)`, the way round
/// the planet either way.
fn near(road: &Road, (x, y): (i64, i64), width: i64) -> bool {
    let reach = WORLD_MAP_SIGHT + WORLD_MAP_PRELOAD;
    let (min, max) = road.bounds();
    let x = x.rem_euclid(width);
    [-width, 0, width].iter().any(|shift| {
        min.0 + shift <= x + reach
            && max.0 + shift >= x - reach
            && min.1 <= y + reach
            && max.1 >= y - reach
    })
}
