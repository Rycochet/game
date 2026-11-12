//! A player's move on a level in units (`ClientMessage::Move`): the cell rule
//! for a step from one cell's middle to another's, otherwise the segment rule.

use super::{Gait, Step, step_on_level_by};
use crate::clearance::{Passage, check_segment};
use crate::creature::Size;
use crate::dungeon::Level;
use crate::protocol::{MOST_MOVE, is_middle};
use crate::spot::Spot;

/// How a legal move on a level goes.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LevelMove {
    /// A step (or jump) from one cell's middle to another's, as the cell rule
    /// allows it (`step_on_level_by`: swimming deep water, World Gates).
    Step { from: (i64, i64), to: (i64, i64) },
    /// Any other straight move the segment rule allows (`check_segment`).
    Segment(Passage),
}

/// Whether a player of `size` with this `gait` may move straight from `from`
/// to `to` on `level`, and how. Never longer than `MOST_MOVE`, never nowhere.
/// Until routes and moves leave cells (free positions PR 4), the cell rule
/// keeps every step it took before allowed, the same as ever.
pub fn move_on_level(
    level: &Level,
    size: Size,
    gait: Gait,
    from: Spot,
    to: Spot,
) -> Option<LevelMove> {
    if from == to || from.distance(to) > MOST_MOVE {
        return None;
    }
    // The cell rule was made for creatures fitting a cell: bigger ones go by
    // their circles alone.
    if size <= Size::Medium && is_middle(from) && is_middle(to) {
        let (start, end) = (from.cell(), to.cell());
        let (dx, dy) = (end.0 - start.0, end.1 - start.1);
        if let Some(step) = Step::new(dx.signum(), dy.signum())
            && step_on_level_by(level, start, step, gait) == Some(end)
        {
            return Some(LevelMove::Step {
                from: start,
                to: end,
            });
        }
    }
    check_segment(level, size, from, to).map(LevelMove::Segment)
}

impl LevelMove {
    /// Whether it jumps over something (liquid, a hole or a fence).
    pub fn jumped(self) -> bool {
        match self {
            Self::Step { from, to } => super::is_jump(from, to),
            Self::Segment(passage) => passage.jumped.is_some(),
        }
    }

    /// How much slower than walking freely, in thousandths: a squeeze through
    /// a gap (`clearance::squeeze_per_mille`); a step by the cell rule never is.
    pub fn squeeze_per_mille(self) -> u32 {
        match self {
            Self::Step { .. } => 1000,
            Self::Segment(passage) => passage.squeeze_per_mille,
        }
    }
}

#[cfg(test)]
mod tests;
