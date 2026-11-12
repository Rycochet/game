//! Players in the Hall of Worlds (`shared::hall`): getting there, and steps
//! along it. Its cells are fixed rules both sides know, so nothing of it is
//! sent but where the player is, and nothing of it is remembered.
//!
//! Where a player is in the Hall is kept in their `CharacterWorldSave::hall`
//! (the world they came from), not wrapped while they are there, so walking
//! over the seam never jumps; it is wrapped when they come back to it.

use bevy::prelude::*;
use shared::creature::Size;
use shared::hall::{self, GateAddress, GatePlace};
use shared::movement::{Gait, LevelMove, move_on_level};
use shared::protocol::ServerMessage;
use shared::spot::{CELL_SPAN, Spot};
use shared::world::WorldSeed;

use crate::connections::Client;
use crate::{ServerWorld, sites};

/// How far along the Hall others can be made out, in tiles: it is all open.
pub(crate) const HALL_SIGHT: i64 = 40;

/// How a move in the Hall from `from` to `to` (the Hall's own units) goes, if
/// it can be taken: by the rule for any level, on the stretch round them.
pub(crate) fn landing(from: Spot, to: Spot, gait: Gait) -> Option<LevelMove> {
    // Room for any move, a jump included.
    let west = from.cell().0.min(to.cell().0) - 2;
    let across = (from.cell().0 - to.cell().0).abs() + 5;
    let around = hall::level(west, across);
    move_on_level(&around, Size::Medium, gait, from, to)
}

/// The player moves in the Hall to `to` (the Hall's own units), if they can (a
/// refused move is left for `Position` to put right). Whether they moved.
pub(crate) fn moved(client: &mut Client, to: Spot) -> bool {
    let Some(from) = client.character.hall else {
        return false;
    };
    let gait = Gait::of(&client.sheet.skills);
    if landing(from, to, gait).is_none() {
        info!("Refused a move in the Hall from {from:?} to {to:?}");
        return false;
    }
    client.character.hall = Some(to);
    client.unsaved = true;
    // Onto a gate's floor: through it, to its planet.
    let cell = to.cell();
    if let Some((seed, entrance)) = crate::gates::hall_gate_leads(cell) {
        if let Some(place) = GatePlace::at(cell.0, cell.1) {
            client.went_through(place.number());
        }
        client.go_to_world(seed, crate::connections::Arrival::Gate(entrance));
    }
    true
}

/// Where the client counts `spot` (the Hall's own units) from: its
/// `hall_origin`.
pub(crate) fn as_sent(client: &Client, spot: Spot) -> Spot {
    spot.offset(-client.hall_origin * CELL_SPAN, 0)
}

/// Puts the player in the Hall, here (the Hall's own units), out of any site.
/// `false` if the client has gone.
pub(crate) fn enter(client: &mut Client, world: &mut ServerWorld, at: Spot) -> bool {
    sites::leave(client, world);
    client.visit = None;
    let (x, y) = at.cell();
    let wrapped = hall::wrap_x(x);
    let at = at.offset((wrapped - x) * CELL_SPAN, 0);
    // A place that can't be walked on (the Hall was once wider): onto the
    // walkway beside it.
    let at = if hall::cell(wrapped, y).is_walkable() {
        at
    } else {
        Spot::middle_of((wrapped, y.clamp(0, hall::WALKWAY_ROWS - 1)))
    };
    client.character.hall = Some(at);
    client.hall_origin = wrapped - wrapped.rem_euclid(hall::GATE_SPACING);
    client.unsaved = true;
    client.changed_map();
    let at = as_sent(client, at);
    client.link.send(&ServerMessage::EnterHall {
        at,
        origin: client.hall_origin,
    })
}

/// Developer tools: to the Hall, in front of the gate leading to World Gate
/// `gate` of the world with this seed (none: the player's own). With no gate,
/// the one nearest the player on their own world, or the first of another.
/// A world that isn't one of the Hall's puts them at the Hall's first gate.
pub(crate) fn to_hall(
    client: &mut Client,
    world: &mut ServerWorld,
    seed: Option<u64>,
    gate: Option<u8>,
) -> bool {
    let to = seed.map_or(world.seed, WorldSeed::new);
    let entrance = gate.or_else(|| {
        if to != world.seed {
            return Some(0);
        }
        let nearest = crate::world_map::nearest_gate(world, client.world_position());
        let gates = &world.pois.world_gates().gates;
        let at = gates.iter().position(|gate| (gate.x, gate.y) == nearest)?;
        u8::try_from(at).ok()
    });
    let place = entrance
        .and_then(|entrance| GateAddress::for_world(to, entrance))
        .map_or(GatePlace::from_number(0), GateAddress::place);
    enter(client, world, Spot::middle_of(place.front()))
}

/// Developer tools: jumps to anywhere on the Hall's floor, given in units as
/// the client counts them (from `Client::hall_origin`).
pub(crate) fn teleport(client: &mut Client, x: i64, y: i64) -> bool {
    let at = Spot::new(x, y).offset(client.hall_origin * CELL_SPAN, 0);
    let (cx, cy) = at.cell();
    if !hall::cell(cx, cy).is_walkable() {
        info!("Refused a teleport off the Hall's floor, to {x}, {y}");
        return true;
    }
    client.character.hall = Some(at);
    client.unsaved = true;
    client.changed_map();
    client.link.send(&ServerMessage::Teleported { x, y })
}

#[cfg(test)]
mod tests;
