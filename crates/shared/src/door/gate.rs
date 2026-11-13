//! Gates: two doors side by side in a wall (`site::Gate`), opened and closed
//! together and drawn as one.

use crate::dungeon::{Cell, Level};

/// The other leaf of the gate that the door at `(x, y)` is in: a door (or
/// secret door) beside it along its wall. Generators never put two other doors
/// side by side (`site::grid::door_nearby`), so any two that are make a gate.
pub fn gate_partner(level: &Level, x: i64, y: i64) -> Option<(i64, i64)> {
    let is_door = |x, y| matches!(level.cell(x, y), Cell::Door | Cell::SecretDoor);
    if !is_door(x, y) {
        return None;
    }
    let (dx, dy) = if level.opening_runs_east_west(x, y) {
        (1, 0)
    } else {
        (0, 1)
    };
    [(x - dx, y - dy), (x + dx, y + dy)]
        .into_iter()
        .find(|&(px, py)| is_door(px, py))
}

/// Of a gate's two leaves, the one its state starts from: the west or north.
pub fn gate_first_leaf(level: &Level, x: i64, y: i64) -> (i64, i64) {
    match gate_partner(level, x, y) {
        Some(other) if other < (x, y) => other,
        _ => (x, y),
    }
}
