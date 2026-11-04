//! The way a road goes: the quickest route by the walking rules (A*), within
//! `MARGIN` of the box its two towns make, with slopes counting less for
//! larger roads.

use std::cmp::Reverse;
use std::collections::BinaryHeap;

use super::{MARGIN, RoadKind};
use crate::movement::{DIAGONAL_COST, SLOPE_COST, STRAIGHT_COST, WorldConditions, WorldGround};

/// The order steps are tried in, so equally quick routes always come out the
/// same (as `pathfinding`'s).
const DIRECTIONS: [(i64, i64); 8] = [
    (0, -1),
    (1, 0),
    (0, 1),
    (-1, 0),
    (1, -1),
    (1, 1),
    (-1, 1),
    (-1, -1),
];

/// The points from `from` to `to`, one step apart and both included, over
/// ground that can be walked in `conditions`, or `None` if water is in the way.
pub(super) fn route(
    ground: &WorldGround,
    conditions: WorldConditions,
    from: (i64, i64),
    to: (i64, i64),
    kind: RoadKind,
) -> Option<Vec<(i64, i64)>> {
    /// Not looked at yet (a height the ground never has, water being `None`).
    const UNKNOWN: Option<i32> = Some(i32::MIN);

    let (min_x, min_y) = (from.0.min(to.0) - MARGIN, from.1.min(to.1) - MARGIN);
    let (max_x, max_y) = (from.0.max(to.0) + MARGIN, from.1.max(to.1) + MARGIN);
    let side_x = max_x - min_x + 1;
    let inside = |(x, y): (i64, i64)| (min_x..=max_x).contains(&x) && (min_y..=max_y).contains(&y);
    let index = |(x, y): (i64, i64)| ((y - min_y) * side_x + (x - min_x)) as usize;
    let len = (side_x * (max_y - min_y + 1)) as usize;

    // Sampling the ground is the slow part, so each point is sampled once.
    let mut elevations = vec![UNKNOWN; len];
    let mut elevation = |point: (i64, i64)| {
        let known = &mut elevations[index(point)];
        if *known == UNKNOWN {
            *known = ground.elevation(point.0, point.1, conditions);
        }
        *known
    };
    let slope_cost = SLOPE_COST * kind.slope_percent() / 100;

    let mut best = vec![u32::MAX; len];
    let mut came_from = vec![from; len];
    best[index(from)] = 0;
    let mut open = BinaryHeap::new();
    open.push(Reverse((estimate(from, to), estimate(from, to), from)));
    while let Some(Reverse((total, left, at))) = open.pop() {
        let cost = total - left;
        if cost > best[index(at)] {
            continue;
        }
        if at == to {
            break;
        }
        let Some(here) = elevation(at) else {
            continue;
        };
        for (dx, dy) in DIRECTIONS {
            let next = (at.0 + dx, at.1 + dy);
            if !inside(next) {
                continue;
            }
            // Roads only cross land: never water, even from water.
            let Some(there) = elevation(next) else {
                continue;
            };
            let distance = if dx != 0 && dy != 0 {
                DIAGONAL_COST
            } else {
                STRAIGHT_COST
            };
            let next_cost = cost + distance + here.abs_diff(there) * slope_cost;
            if next_cost >= best[index(next)] {
                continue;
            }
            best[index(next)] = next_cost;
            came_from[index(next)] = at;
            let left = estimate(next, to);
            open.push(Reverse((next_cost + left, left, next)));
        }
    }
    if best[index(to)] == u32::MAX {
        return None;
    }
    let mut points = vec![to];
    let mut at = to;
    while at != from {
        at = came_from[index(at)];
        points.push(at);
    }
    points.reverse();
    Some(points)
}

/// The least a route between two points can cost: as the crow flies, with no
/// slopes. Never more than the real cost, so the route found is the quickest.
fn estimate(from: (i64, i64), to: (i64, i64)) -> u32 {
    let (dx, dy) = ((to.0 - from.0).abs(), (to.1 - from.1).abs());
    let (long, short) = (dx.max(dy) as u32, dx.min(dy) as u32);
    (long - short) * STRAIGHT_COST + short * DIAGONAL_COST
}
