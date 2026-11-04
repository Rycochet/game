//! Players in the Hall of Worlds (`shared::hall`): getting there, and steps
//! along it. Its cells are fixed rules both sides know, so nothing of it is
//! sent but where the player is, and nothing of it is remembered.
//!
//! Where a player is in the Hall is kept in their `CharacterWorldSave::hall`
//! (the world they came from), not wrapped while they are there, so walking
//! over the seam never jumps; it is wrapped when they come back to it.

use bevy::prelude::*;
use shared::hall::{self, GateAddress, GatePlace};
use shared::movement::{Step, step_on_level};
use shared::protocol::ServerMessage;

use crate::connections::Client;
use crate::{ServerWorld, sites};

/// How far along the Hall others can be made out, in tiles: it is all open.
pub(crate) const HALL_SIGHT: i64 = 40;

/// Where a step from here in the Hall lands, if it can be taken.
pub(crate) fn landing(from: (i64, i64), step: Step) -> Option<(i64, i64)> {
    // Room for any step, a jump included.
    let around = hall::level(from.0 - 2, 5);
    step_on_level(&around, from, step)
}

/// The player takes a step in the Hall, if it can be taken (a refused step is
/// left for `Position` to put right). Always stays.
pub(crate) fn stepped(client: &mut Client, step: Step) -> bool {
    let Some(from) = client.character.hall else {
        return true;
    };
    match landing(from, step) {
        Some(to) => {
            client.character.hall = Some(to);
            client.unsaved = true;
        }
        None => info!("Refused a step in the Hall from {from:?}"),
    }
    true
}

/// Puts the player in the Hall, here, out of any site. `false` if the client
/// has gone.
pub(crate) fn enter(client: &mut Client, world: &mut ServerWorld, (x, y): (i64, i64)) -> bool {
    sites::leave(client, world);
    client.visit = None;
    let x = hall::wrap_x(x);
    // A place that can't be walked on (the Hall was once wider): onto the
    // walkway beside it.
    let y = if hall::cell(x, y).is_walkable() {
        y
    } else {
        y.clamp(0, hall::WALKWAY_ROWS - 1)
    };
    client.character.hall = Some((x, y));
    client.hall_origin = x - x.rem_euclid(hall::GATE_SPACING);
    client.unsaved = true;
    client.changed_map();
    client.link.send(&ServerMessage::EnterHall {
        x: x - client.hall_origin,
        y,
        origin: client.hall_origin,
    })
}

/// Developer tools: to the Hall, in front of the gate leading to the World Gate
/// nearest the player, or of its first gate if this planet isn't one of the
/// Hall's.
pub(crate) fn to_hall(client: &mut Client, world: &mut ServerWorld) -> bool {
    let nearest = crate::world_map::nearest_gate(world, client.world_position());
    let entrance = world
        .pois
        .world_gates()
        .gates
        .iter()
        .position(|gate| (gate.x, gate.y) == nearest);
    let place = entrance
        .and_then(|entrance| GateAddress::for_world(world.seed, entrance as u8))
        .map_or(GatePlace::from_number(0), GateAddress::place);
    enter(client, world, place.front())
}

/// Developer tools: jumps to a walkable tile of the Hall, given as the
/// client counts it (from `Client::hall_origin`).
pub(crate) fn teleport(client: &mut Client, x: i64, y: i64) -> bool {
    let origin = client.hall_origin;
    if !hall::cell(x + origin, y).is_walkable() {
        info!("Refused a teleport off the Hall's floor, to {x}, {y}");
        return true;
    }
    client.character.hall = Some((x + origin, y));
    client.unsaved = true;
    client.changed_map();
    client.link.send(&ServerMessage::Teleported { x, y })
}

#[cfg(test)]
mod tests;
