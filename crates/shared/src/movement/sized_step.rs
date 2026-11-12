//! A mob's step from one cell to the next for its size: mobs think in cells,
//! but their bodies are circles (free positions, PR 5).

use super::{Step, mob_step_on_level};
use crate::clearance::check_segment;
use crate::creature::Size;
use crate::dungeon::Level;
use crate::spot::Spot;

/// Where a mob of `size` stepping `step` from cell `from` lands, and how much
/// slower than walking freely, in thousandths (`clearance::squeeze_per_mille`).
/// The cell rule first (`mob_step_on_level`: never onto a World Gate); then
/// anyone bigger than Medium must also fit, its circle going from one cell's
/// middle to the other's (`check_segment`): a Large mob squeezes through a
/// doorway, a Giant never gets through one.
pub fn mob_step_sized(
    level: &Level,
    size: Size,
    from: (i64, i64),
    step: Step,
) -> Option<((i64, i64), u32)> {
    let to = mob_step_on_level(level, from, step)?;
    if size <= Size::Medium {
        return Some((to, 1000));
    }
    let passage = check_segment(level, size, Spot::middle_of(from), Spot::middle_of(to))?;
    Some((to, passage.squeeze_per_mille))
}
