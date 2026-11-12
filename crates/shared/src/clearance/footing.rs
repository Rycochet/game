//! Where a creature's middle may stand.

use crate::dungeon::{Cell, Level};

/// Whether a creature's middle may stand in cell `(x, y)`: walkable ground, or
/// a fence's or tree's cell beside the fence or trunk (they block only where
/// they stand: `cell_blockers`). Liquid and holes can only be jumped.
pub fn has_footing(level: &Level, x: i64, y: i64) -> bool {
    level.is_walkable(x, y) || matches!(level.cell(x, y), Cell::Fence | Cell::Tree)
}
