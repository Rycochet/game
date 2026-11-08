//! A found route: its cells and the steps along it.

use crate::movement::Step;

/// A route across a level.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Path {
    /// The steps to take, in order.
    pub steps: Vec<Step>,
    /// Where each step lands (two cells on for a jump over liquid). The
    /// starting point isn't included.
    pub cells: Vec<(i64, i64)>,
    /// How long it takes to walk, in `movement::step_cost` units (with any
    /// terrain's cost, for `find_path_by`).
    pub cost: u32,
    /// Whether it ends on the goal. If not, it ends on the cell nearest the
    /// goal that could be reached (the start, with no steps, if that is it).
    pub reached: bool,
}

impl Path {
    /// The path through these cells, each a step on from the one before,
    /// priced by `cost_of`.
    pub(super) fn through(
        from: (i64, i64),
        cells: Vec<(i64, i64)>,
        reached: bool,
        cost_of: &impl Fn((i64, i64), (i64, i64)) -> u32,
    ) -> Self {
        let mut at = from;
        let mut steps = Vec::with_capacity(cells.len());
        let mut cost = 0;
        for &cell in &cells {
            let step = Step::new((cell.0 - at.0).signum(), (cell.1 - at.1).signum())
                .expect("cells of a path differ");
            steps.push(step);
            cost += cost_of(at, cell);
            at = cell;
        }
        Self {
            steps,
            cells,
            cost,
            reached,
        }
    }
}
