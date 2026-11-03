//! Spreading a route's diagonals out along straighter lines.

use super::Path;
use crate::dungeon::Level;
use crate::movement::{Step, step_cost, step_on_level};

/// The same route, as quick and as legal, with its diagonals spread out along
/// each stretch instead of bunched together, which looks like walking in a
/// line. A separate step because only what is drawn cares: anything that
/// recalculates often can skip it. Jumps stay where they are, with each
/// stretch between them straightened alone.
pub fn straighten(
    level: &Level,
    from: (i64, i64),
    path: &Path,
    allowed: impl Fn(i64, i64) -> bool,
) -> Path {
    let mut points = vec![from];
    points.extend_from_slice(&path.cells);

    // The cost of the route up to each point, to compare lines against.
    let mut cost_to = vec![0];
    for pair in points.windows(2) {
        cost_to.push(cost_to[cost_to.len() - 1] + step_cost(pair[0], pair[1]));
    }
    let is_jump = |a: (i64, i64), b: (i64, i64)| (b.0 - a.0).abs() > 1 || (b.1 - a.1).abs() > 1;

    let mut cells = Vec::with_capacity(path.cells.len());
    let mut anchor = 0;
    while anchor + 1 < points.len() {
        // Where this stretch stops: the next jump, or the end.
        let stretch_end = (anchor..points.len() - 1)
            .find(|&i| is_jump(points[i], points[i + 1]))
            .unwrap_or(points.len() - 1);
        if stretch_end == anchor {
            cells.push(points[anchor + 1]);
            anchor += 1;
            continue;
        }
        // The furthest point along it that a line can replace the route to.
        let replaced = (anchor + 2..=stretch_end).rev().find_map(|end| {
            let line = line_cells(points[anchor], points[end]);
            let budget = cost_to[end] - cost_to[anchor];
            line_is_as_good(level, points[anchor], &line, budget, &allowed).then_some((end, line))
        });
        match replaced {
            Some((end, line)) => {
                cells.extend(line);
                anchor = end;
            }
            None => {
                cells.push(points[anchor + 1]);
                anchor += 1;
            }
        }
    }
    Path::through(from, cells, path.reached)
}

/// The fewest-steps line from one point to another, without the first: as many
/// diagonals as it takes, spread evenly among the straight steps.
pub(super) fn line_cells(from: (i64, i64), to: (i64, i64)) -> Vec<(i64, i64)> {
    let (dx, dy) = (to.0 - from.0, to.1 - from.1);
    let long = dx.abs().max(dy.abs());
    let short = dx.abs().min(dy.abs());
    (1..=long)
        .map(|k| {
            // `k * short / long`, rounded to nearest, in whole numbers.
            let across = (2 * k * short + long) / (2 * long);
            if dx.abs() >= dy.abs() {
                (from.0 + k * dx.signum(), from.1 + across * dy.signum())
            } else {
                (from.0 + across * dx.signum(), from.1 + k * dy.signum())
            }
        })
        .collect()
}

/// Whether a line can be walked, one cell a step, and costs exactly `budget`.
pub(super) fn line_is_as_good(
    level: &Level,
    from: (i64, i64),
    line: &[(i64, i64)],
    budget: u32,
    allowed: &impl Fn(i64, i64) -> bool,
) -> bool {
    let mut at = from;
    let mut cost = 0;
    for &cell in line {
        let Some(step) = Step::new(cell.0 - at.0, cell.1 - at.1) else {
            return false;
        };
        if step_on_level(level, at, step) != Some(cell) || !allowed(cell.0, cell.1) {
            return false;
        }
        cost += step_cost(at, cell);
        at = cell;
    }
    cost == budget
}
