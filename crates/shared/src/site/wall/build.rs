//! Putting a wall on a level.

use super::WallLook;
use crate::dungeon::{Cell, Level};

/// Sets a wall's cells on `level`: wall in no room, and each gate's two doors.
/// Gates are put straight in, side by side, so they are never held back by
/// the rule that keeps other doors apart (`grid::door_nearby`).
pub(in crate::site) fn build_wall(level: &mut Level, wall: &WallLook) {
    let rect = wall.rect;
    for y in rect.y..rect.y + rect.height {
        for x in rect.x..rect.x + rect.width {
            if wall.has(x, y) {
                level.set(x, y, Cell::Wall);
                level.set_room(x, y, None);
            }
        }
    }
    for gate in &wall.gates {
        for (x, y) in gate.cells() {
            level.set(x, y, Cell::Door);
        }
    }
}
