//! A step asked for on a level or in the Hall (`TakeStep`: a tap of a key's
//! worth from dev tools or the debug menu), as one move in units.

use shared::clearance::check_segment;
use shared::creature::Size;
use shared::dungeon::Level;
use shared::movement::{Gait, Step, step_on_level_by};
use shared::protocol::{MOST_MOVE, is_middle};
use shared::spot::Spot;

/// Where a step from `from` lands on `level`: the middle of the cell the
/// cell rule takes it to (`step_on_level_by`: a jump over liquid, swimming
/// for a good swimmer), as the server takes a move between two middles; from
/// anywhere else (or for anyone bigger than Medium), the middle of the next cell that way, if the one rule
/// allows walking there straight. `None` if it can't be taken.
pub(super) fn level_step(
    level: &Level,
    size: Size,
    gait: Gait,
    from: Spot,
    step: Step,
) -> Option<Spot> {
    let cell = from.cell();
    if size <= Size::Medium && is_middle(from) {
        return step_on_level_by(level, cell, step, gait).map(Spot::middle_of);
    }
    let to = Spot::middle_of(step.from(cell));
    let legal = from.distance(to) <= MOST_MOVE
        && check_segment(level, size, from, to).is_some_and(|p| p.jumped.is_none());
    legal.then_some(to)
}
