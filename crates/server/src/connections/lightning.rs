//! Telling players when lightning flashes where they are, and where bolts
//! strike near enough to see: looked for once a second of the server's clock,
//! so not while it is paused, and at once for anyone who asked (developer
//! tools).

use bevy::prelude::*;
use shared::protocol::{ServerMessage, Strike, WORLD_MAP_SIGHT};
use shared::wilderness::SCALE;
use std::collections::BTreeMap;

use super::systems::for_each_world;
use super::{Client, Connections, Phase};
use crate::server_world::apart;
use crate::{ServerWorld, ServerWorlds};

/// How near a player a bolt in their area strikes, in cells each way: in the
/// wilderness within what they can see, on the world map within a few points.
const WILD_REACH: i64 = 20;
const MAP_REACH: i64 = 6 * SCALE;

/// How far a bolt keeps from a player, in cells: a few in the wilderness, and
/// on the world map off the point they stand on.
const WILD_CLEAR: i64 = 3;
const MAP_CLEAR: i64 = SCALE;

/// Every tick: on each new whole second of the server's clock, tells everyone
/// playing on a world (on its map, in its wilderness or in a site, by where
/// they are on its map) if lightning flashes there, and of bolts within sight.
/// Nobody in the Hall of Worlds, which has no sky.
pub(crate) fn tell_of_lightning(
    mut connections: ResMut<Connections>,
    mut worlds: ResMut<ServerWorlds>,
) {
    for_each_world(&mut connections.clients, &mut worlds, |clients, world| {
        let seconds = world.server_clock.seconds();
        let second = seconds.floor() as i64;
        if second != world.lightning_second {
            world.lightning_second = second;
            tell(clients, world, second, true);
        } else if !world.lightning_asked.is_empty() {
            // Asked for: a spot of its own, not the second's.
            tell(clients, world, (seconds * 1000.0) as i64, false);
        }
    });
}

/// Tells everyone under the sky whom lightning flashes for (by the weather if
/// `natural`, and anyone who asked), with a bolt in each flashing area, and of
/// bolts in other areas they can see. `salt` decides the bolts' spots.
pub(super) fn tell(clients: &mut [Client], world: &mut ServerWorld, salt: i64, natural: bool) {
    let asked = std::mem::take(&mut world.lightning_asked);
    let under: Vec<usize> = (0..clients.len())
        .filter(|&i| under_sky(&clients[i]))
        .collect();
    let cells: Vec<(i64, i64, i64)> = under.iter().map(|&i| cell_of(&clients[i])).collect();
    // Each client's area, if it flashes there, and each such area's bolt.
    let mut flashes: Vec<Option<(i64, i64)>> = Vec::new();
    let mut strikes: BTreeMap<(i64, i64), Option<Strike>> = BTreeMap::new();
    for (&i, &(cx, cy, clear)) in under.iter().zip(&cells) {
        let (x, y) = clients[i].world_position();
        let asks = asked.contains(&clients[i].entity_id());
        let flash = asks || (natural && world.lightning_at(x, y, salt));
        let area = world.lightning_area(x, y);
        if flash {
            // Near the first to see it there.
            let reach = if clear == WILD_CLEAR {
                WILD_REACH
            } else {
                MAP_REACH
            };
            strikes
                .entry(area)
                .or_insert_with(|| world.strike_near((cx, cy), reach, salt, &cells));
        }
        flashes.push(flash.then_some(area));
    }
    for (&i, own) in under.iter().zip(flashes) {
        let (x, y) = clients[i].world_position();
        let link = &clients[i].link;
        if let Some(area) = own {
            let strike = strikes[&area].filter(|&strike| in_sight(world, (x, y), strike));
            link.send(&ServerMessage::Lightning {
                flash: true,
                strike,
            });
        }
        for (_, &strike) in strikes.iter().filter(|(area, _)| Some(**area) != own) {
            if let Some(strike) = strike.filter(|&strike| in_sight(world, (x, y), strike)) {
                let strike = Some(strike);
                link.send(&ServerMessage::Lightning {
                    flash: false,
                    strike,
                });
            }
        }
    }
}

/// Whether a client is playing somewhere with weather.
fn under_sky(client: &Client) -> bool {
    let in_hall = client.visit.is_none() && client.character.hall.is_some();
    client.phase == Phase::Playing && !in_hall
}

/// Where a client is in wilderness cells, and how far a bolt keeps from
/// them: their own cell in the wilderness, otherwise the middle cell of their
/// world map point.
fn cell_of(client: &Client) -> (i64, i64, i64) {
    match &client.visit {
        Some(visit) if visit.is_wilderness() => {
            let (x, y) = visit.position();
            (x, y, WILD_CLEAR)
        }
        _ => {
            let (x, y) = client.world_position();
            (x * SCALE + SCALE / 2, y * SCALE + SCALE / 2, MAP_CLEAR)
        }
    }
}

/// Whether a bolt is near enough to a world map point to be drawn there.
fn in_sight(world: &ServerWorld, (x, y): (i64, i64), strike: Strike) -> bool {
    let (sx, sy) = (strike.x.div_euclid(SCALE), strike.y.div_euclid(SCALE));
    apart(sx - x, world.width).max((sy - y).abs()) <= WORLD_MAP_SIGHT
}

#[cfg(test)]
mod tests;
