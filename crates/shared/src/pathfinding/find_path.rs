//! The quickest route across one level (A*).

use std::cmp::Reverse;
use std::collections::BinaryHeap;

use super::{DIRECTIONS, Path, distance_squared, estimate};
use crate::dungeon::Level;
use crate::movement::{Gait, Step, step_cost, step_on_level_by};

/// The quickest route from `from` to `to`, using only legal steps that land on
/// cells `allowed` accepts (the client: cells the player knows; a monster: all
/// of them). `max_cost` stops a search going further than that, which also
/// keeps it cheap. If the goal can't be reached, the route goes to the nearest
/// cell that can, straight-line, so check `reached`.
///
/// Among equally quick routes the diagonals fall anywhere: see `straighten`.
pub fn find_path(
    level: &Level,
    from: (i64, i64),
    to: (i64, i64),
    max_cost: Option<u32>,
    allowed: impl Fn(i64, i64) -> bool,
) -> Path {
    find_path_by(
        level,
        from,
        to,
        max_cost,
        allowed,
        step_cost,
        Gait::UNSKILLED,
    )
}

/// `find_path` with steps priced by `cost_of` (from, to) instead of their
/// distance alone, such as the wilderness's slopes and wading
/// (`movement::level_step_cost`), for someone with this `gait`: a good
/// swimmer steps into water (`movement::step_on_level_by`), and may wade
/// quicker than they walk, so the search's estimate is scaled by that (or it
/// would be too high and routes not the quickest): `cost_of` is never less than
/// `Gait::swim` of `step_cost`, nor `step_cost` itself for anyone slower.
pub fn find_path_by(
    level: &Level,
    from: (i64, i64),
    to: (i64, i64),
    max_cost: Option<u32>,
    allowed: impl Fn(i64, i64) -> bool,
    cost_of: impl Fn((i64, i64), (i64, i64)) -> u32,
    gait: Gait,
) -> Path {
    let cheapest = gait.swim_per_mille();
    let least = |at: (i64, i64)| estimate(at, to) * cheapest.min(1000) / 1000;
    let (min_x, min_y, width, height) = level.bounds();
    let index = |(x, y): (i64, i64)| ((y - min_y) * width + (x - min_x)) as usize;
    if !level.contains(from.0, from.1) {
        return Path::default();
    }

    // For each cell: the cheapest way found to it, and the cell it came from.
    let mut best = vec![u32::MAX; (width * height) as usize];
    let mut came_from = vec![from; best.len()];
    best[index(from)] = 0;

    // Cells to look at, cheapest total first. (`BinaryHeap` gives the largest
    // first, and `Reverse` turns that round.) The rest of the tuple settles
    // ties: closest to the goal, then by position.
    let mut open = BinaryHeap::new();
    open.push(Reverse((least(from), least(from), from)));
    let mut nearest = (distance_squared(from, to), from);

    while let Some(Reverse((total, left, at))) = open.pop() {
        let cost = total - left;
        // Reached again more cheaply since this was added.
        if cost > best[index(at)] {
            continue;
        }
        nearest = nearest.min((distance_squared(at, to), at));
        if at == to {
            break;
        }
        for (dx, dy) in DIRECTIONS {
            let step = Step::new(dx, dy).expect("directions are steps");
            let Some(next) = step_on_level_by(level, at, step, gait) else {
                continue;
            };
            let next_cost = cost + cost_of(at, next);
            if next_cost >= best[index(next)]
                || max_cost.is_some_and(|max| next_cost > max)
                || !allowed(next.0, next.1)
            {
                continue;
            }
            best[index(next)] = next_cost;
            came_from[index(next)] = at;
            let left = least(next);
            open.push(Reverse((next_cost + left, left, next)));
        }
    }

    let end = nearest.1;
    let mut cells = Vec::new();
    let mut at = end;
    while at != from {
        cells.push(at);
        at = came_from[index(at)];
    }
    cells.reverse();
    Path::through(from, cells, end == to, &cost_of)
}
