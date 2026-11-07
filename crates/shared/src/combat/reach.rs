//! Whether an attack can reach its target from where the attacker stands.

use crate::dungeon::{Level, line_is_clear};

/// Whether `to` is within `reach` cells of `from` (diagonals counting as one),
/// with nothing between them a weapon couldn't pass: the line sight goes by.
pub fn in_reach(level: &Level, from: (i64, i64), to: (i64, i64), reach: u32) -> bool {
    let far = (to.0 - from.0).abs().max((to.1 - from.1).abs());
    far >= 1 && far <= i64::from(reach) && line_is_clear(level, from, to)
}
