//! The quickest route across the world map (A*).

use std::cmp::Reverse;
use std::collections::BinaryHeap;

use super::{DIRECTIONS, Path, distance_squared, estimate};
use crate::movement::{Gait, Step, WorldConditions, WorldGround, world_step_cost};
use crate::road::{FASTEST_PERCENT, RoadSteps};

/// How far a route on the world map may go from where it starts, each way, in
/// points. More than the view shows, and the ground is infinite, so this is what
/// keeps the search's cost bounded.
pub const WORLD_ROUTE_REACH: i64 = 320;

/// The quickest route across the world map in these `conditions`: legal
/// steps, slopes costing extra, steps along `roads` less, slopes by `gait`'s Climbing. If the goal can't be reached within
/// `WORLD_ROUTE_REACH`, the route goes to the nearest point that can,
/// straight-line, so check `reached`.
pub fn find_world_path(
    ground: &WorldGround,
    conditions: WorldConditions,
    roads: &RoadSteps,
    from: (i64, i64),
    to: (i64, i64),
    gait: Gait,
) -> Path {
    /// Not looked at yet (a height the ground never has, water being `None`).
    const UNKNOWN: Option<i32> = Some(i32::MIN);

    let side = 2 * WORLD_ROUTE_REACH + 1;
    let (min_x, min_y) = (from.0 - WORLD_ROUTE_REACH, from.1 - WORLD_ROUTE_REACH);
    let inside = |(x, y): (i64, i64)| {
        (x - from.0).abs() <= WORLD_ROUTE_REACH && (y - from.1).abs() <= WORLD_ROUTE_REACH
    };
    let index = |(x, y): (i64, i64)| ((y - min_y) * side + (x - min_x)) as usize;

    // Sampling the ground is the slow part, so each point is only sampled once.
    let mut elevations = vec![UNKNOWN; (side * side) as usize];
    let mut elevation = |point: (i64, i64)| {
        let known = &mut elevations[index(point)];
        if *known == UNKNOWN {
            *known = ground.elevation(point.0, point.1, conditions);
        }
        *known
    };

    // As in `find_path`.
    let mut best = vec![u32::MAX; (side * side) as usize];
    let mut came_from = vec![from; best.len()];
    best[index(from)] = 0;
    // With roads about, the least a route can cost is at the fastest road's
    // speed, or the route found may not be the quickest.
    let least = |at: (i64, i64)| {
        if roads.is_empty() {
            estimate(at, to)
        } else {
            estimate(at, to) * 100 / FASTEST_PERCENT
        }
    };
    let mut open = BinaryHeap::new();
    open.push(Reverse((least(from), least(from), from)));
    let mut nearest = (distance_squared(from, to), from);

    while let Some(Reverse((total, left, at))) = open.pop() {
        let cost = total - left;
        if cost > best[index(at)] {
            continue;
        }
        nearest = nearest.min((distance_squared(at, to), at));
        if at == to {
            break;
        }
        let here = elevation(at);
        for (dx, dy) in DIRECTIONS {
            let next = (at.0 + dx, at.1 + dy);
            if !inside(next) {
                continue;
            }
            // The rule of `step_on_world_map`, with the samples kept.
            let there = elevation(next);
            if (here.is_some() && there.is_none()) || ground.past_the_ice(next.1) {
                continue;
            }
            let road = roads.kind_of(at, next);
            let next_cost = cost + world_step_cost(at, next, (here, there), road, gait);
            if next_cost >= best[index(next)] {
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
    let mut steps = Vec::with_capacity(cells.len());
    let mut at = from;
    for &cell in &cells {
        steps.push(Step::new(cell.0 - at.0, cell.1 - at.1).expect("cells of a path differ"));
        at = cell;
    }
    Path {
        steps,
        cells,
        cost: best[index(end)],
        reached: end == to,
    }
}
