//! Where a creature's middle may stand.

use crate::dungeon::{Cell, Level};

/// Whether a creature's middle may stand in cell `(x, y)`: walkable ground, or
/// a fence's or tree's cell beside the fence or trunk (they block only where
/// they stand: `cell_blockers`). Liquid and holes can only be jumped.
pub fn has_footing(level: &Level, x: i64, y: i64) -> bool {
    level.is_walkable(x, y) || matches!(level.cell(x, y), Cell::Fence | Cell::Tree)
}

/// `has_footing` for someone who may swim deep water (`Gait::swims_deep`):
/// water too for them (never lava), as `movement::step_on_level_by` allows.
pub fn has_footing_by(level: &Level, swims: bool, x: i64, y: i64) -> bool {
    has_footing(level, x, y) || (swims && level.cell(x, y) == Cell::Water)
}
