//! The one rule for a move on a level: a creature's circle going straight from
//! one spot to another.

use super::{Blocker, blockers_near, has_footing_by, squeeze_per_mille};
use crate::creature::Size;
use crate::dungeon::{Level, line_hits_box};
use crate::spot::{HALF_SPAN, Spot};

/// The longest jump, in units: two cells diagonally (as far as a jump went
/// on whole cells).
pub const JUMP_REACH: u64 = 679;

/// How a legal move goes.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Passage {
    /// How much slower than walking freely, in thousandths (`squeeze`).
    pub squeeze_per_mille: u32,
    /// The cell jumped over (liquid, a hole or a fence), if it is a jump.
    pub jumped: Option<(i64, i64)>,
}

/// Whether a creature of `size` may move straight from `from` to `to`, and how
/// (`None`: it may not). Both ends need footing (`has_footing`); the middle's
/// line may cross no cell without footing unless the move is a jump: no longer
/// than `JUMP_REACH`, over one cell of liquid, holes or fence and nothing else.
/// The circle, swept along, must keep at least half its radius from every
/// blocker but the fence it jumps (squeezing where it has less than its
/// radius).
pub fn check_segment(level: &Level, size: Size, from: Spot, to: Spot) -> Option<Passage> {
    check_segment_by(level, size, false, from, to)
}

/// `check_segment` for someone who may or may not swim deep water: for a
/// swimmer, water is footing (`has_footing_by`), swum rather than jumped.
pub fn check_segment_by(
    level: &Level,
    size: Size,
    swims: bool,
    from: Spot,
    to: Spot,
) -> Option<Passage> {
    let footing = |x, y| has_footing_by(level, swims, x, y);
    let (fx, fy) = from.cell();
    let (tx, ty) = to.cell();
    if !footing(fx, fy) || !footing(tx, ty) {
        return None;
    }
    let low = Spot::new(from.x.min(to.x), from.y.min(to.y));
    let high = Spot::new(from.x.max(to.x), from.y.max(to.y));
    let mut jumped = crossed_without_footing(level, &footing, from, to, low, high)?;

    let radius = size.radius();
    let mut clearance = radius;
    let blockers: Vec<Blocker> = blockers_near(level, low, high, radius);
    // Crossing a fence is jumping it.
    for blocker in &blockers {
        if let Some(cell) = blocker.fence
            && blocker.shape.clearance(from, to) == 0
        {
            if jumped.is_some_and(|j| j != cell) {
                return None;
            }
            jumped = Some(cell);
        }
    }
    for blocker in &blockers {
        if blocker.fence.is_some() && blocker.fence == jumped {
            continue;
        }
        clearance = clearance.min(blocker.shape.clearance(from, to));
    }
    if jumped.is_some() && from.distance(to) > JUMP_REACH {
        return None;
    }
    Some(Passage {
        squeeze_per_mille: squeeze_per_mille(radius, clearance)?,
        jumped,
    })
}

/// `check_segment` as a yes or no.
pub fn segment_is_legal(level: &Level, size: Size, from: Spot, to: Spot) -> bool {
    check_segment(level, size, from, to).is_some()
}

/// What a move takes, in units of distance walked freely: its length, longer
/// by any squeeze. `None` if it isn't legal.
pub fn segment_cost(level: &Level, size: Size, from: Spot, to: Spot) -> Option<u64> {
    let passage = check_segment(level, size, from, to)?;
    Some(from.distance(to) * passage.squeeze_per_mille as u64 / 1000)
}

/// The one jumpable cell without footing whose inside the middle's line
/// crosses, if any (`Some(None)`: none); `None` if it crosses more than one, or
/// any that can't be jumped.
fn crossed_without_footing(
    level: &Level,
    footing: &impl Fn(i64, i64) -> bool,
    from: Spot,
    to: Spot,
    low: Spot,
    high: Spot,
) -> Option<Option<(i64, i64)>> {
    let (min_x, min_y) = low.cell();
    let (max_x, max_y) = high.cell();
    let mut jumped = None;
    for y in min_y..=max_y {
        for x in min_x..=max_x {
            if footing(x, y) {
                continue;
            }
            // The inside only: a line along an edge or through a corner
            // doesn't cross the cell.
            let middle = Spot::middle_of((x, y));
            let inner = HALF_SPAN - 1;
            let (a, b) = (middle.offset(-inner, -inner), middle.offset(inner, inner));
            if !line_hits_box((from.x, from.y), (to.x, to.y), (a.x, a.y), (b.x, b.y)) {
                continue;
            }
            if !level.cell(x, y).is_jumpable() || jumped.is_some() {
                return None;
            }
            jumped = Some((x, y));
        }
    }
    Some(jumped)
}
