//! Gates (`shared::door::gate_partner`): a gate's two doors open and close
//! together. A broken leaf stays as it is, and breaking one leaves the other.

use shared::door::{Swing, gate_partner};

use super::act::set_here;
use super::{VisitDoor, keep_door};
use crate::ServerWorld;
use crate::connections::Client;
use crate::sites::Visit;

/// Sets a door on the visit's level and keeps it (`set_here`, `keep_door`),
/// and the other leaf of its gate with it, if it is in one: opened the same
/// way, or closed.
pub(super) fn set_with_gate(
    visit: &mut Visit,
    world: &mut ServerWorld,
    at: (i64, i64),
    door: VisitDoor,
) {
    set_here(visit, at, door);
    keep_door(world, door.key, door.door);
    let Some(other) = gate_partner(&visit.level.level, at.0, at.1) else {
        return;
    };
    let Some(mut leaf) = visit.doors.get(&other).copied() else {
        return;
    };
    let broken = |swing: Swing| matches!(swing, Swing::Broken(_));
    if broken(door.door.swing) || broken(leaf.door.swing) || leaf.door.swing == door.door.swing {
        return;
    }
    leaf.door.swing = door.door.swing;
    set_here(visit, other, leaf);
    keep_door(world, leaf.key, leaf.door);
}

/// Whether someone stands in the doorway of the other leaf of the gate the
/// door at `at` is in, so neither can be closed.
pub(super) fn gate_blocked(client: &Client, world: &mut ServerWorld, at: (i64, i64)) -> bool {
    let Some(visit) = &client.visit else {
        return false;
    };
    let Some(other) = gate_partner(&visit.level.level, at.0, at.1) else {
        return false;
    };
    let Some(leaf) = visit.doors.get(&other).copied() else {
        return false;
    };
    matches!(leaf.door.swing, Swing::Open(_)) && world.someone_in_doorway(client, other, leaf.key)
}
