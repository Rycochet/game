//! Joining caves with winding passages.

use super::super::grid::{Reach, chance, roll};
use super::{Cave, MAX_RUN, MIN_RUN, WANDER_PERCENT};
use crate::dungeon::{Cell, Level};
use crate::rng::Rng;

/// Joins every cave to the nearest one already joined, then adds a few
/// more passages for loops.
pub(super) fn join_caves(
    level: &mut Level,
    rng: &mut Rng,
    caves: &[Cave],
    bounds: (i64, i64, i64, i64),
) {
    let distance = |a: &Cave, b: &Cave| (a.x - b.x).pow(2) + (a.y - b.y).pow(2);
    let mut joined = vec![false; caves.len()];
    joined[0] = true;
    for _ in 1..caves.len() {
        let mut best: Option<(i64, usize, usize)> = None;
        for (a, cave_a) in caves.iter().enumerate().filter(|(a, _)| joined[*a]) {
            for (b, cave_b) in caves.iter().enumerate().filter(|(b, _)| !joined[*b]) {
                let d = distance(cave_a, cave_b);
                if best.is_none_or(|(best_d, _, _)| d < best_d) {
                    best = Some((d, a, b));
                }
            }
        }
        let Some((_, a, b)) = best else { break };
        joined[b] = true;
        wind(
            level,
            rng,
            (caves[a].x, caves[a].y),
            (caves[b].x, caves[b].y),
            bounds,
        );
    }
    // Loops: some caves get a passage to their second nearest.
    for (a, cave_a) in caves.iter().enumerate() {
        if caves.len() < 3 || !chance(rng, 25) {
            continue;
        }
        let mut others: Vec<(i64, usize)> = caves
            .iter()
            .enumerate()
            .filter(|(b, _)| *b != a)
            .map(|(b, cave_b)| (distance(cave_a, cave_b), b))
            .collect();
        others.sort();
        let b = others[1].1;
        wind(
            level,
            rng,
            (cave_a.x, cave_a.y),
            (caves[b].x, caves[b].y),
            bounds,
        );
    }
}

/// Digs a winding passage from `from` to `to`: short straight stretches,
/// mostly towards the target, sometimes wandering sideways. Stays within
/// `bounds` (`min_x, min_y, max_x, max_y`, inclusive). New cells are floor
/// in no room.
pub(super) fn wind(
    level: &mut Level,
    rng: &mut Rng,
    from: (i64, i64),
    to: (i64, i64),
    bounds: (i64, i64, i64, i64),
) {
    let (min_x, min_y, max_x, max_y) = bounds;
    let dig = |level: &mut Level, x: i64, y: i64| {
        if level.cell(x, y) == Cell::Wall {
            level.set(x, y, Cell::Floor);
            level.set_room(x, y, None);
        }
    };
    let (mut x, mut y) = from;
    dig(level, x, y);
    let most_runs = 4 * ((to.0 - x).abs() + (to.1 - y).abs()) / MIN_RUN + 8;
    // Gives up after twice the expected runs (if the target is out of bounds).
    for run in 0..most_runs * 2 {
        if (x, y) == to {
            return;
        }
        let (gx, gy) = (to.0 - x, to.1 - y);
        let wander = run < most_runs && chance(rng, WANDER_PERCENT);
        // Towards the target along one axis (the further, more often), or
        // sideways off it.
        let along_x = if gx == 0 {
            false
        } else if gy == 0 {
            true
        } else {
            rng.range_u64(0, gx.unsigned_abs() + gy.unsigned_abs()) < gx.unsigned_abs()
        };
        let (dx, dy, length) = if wander {
            let sign = if chance(rng, 50) { 1 } else { -1 };
            let length = roll(rng, MIN_RUN, MAX_RUN / 2);
            if along_x {
                (0, sign, length)
            } else {
                (sign, 0, length)
            }
        } else if along_x {
            let length = roll(rng, MIN_RUN, MAX_RUN).min(gx.abs());
            (
                gx.signum(),
                0,
                if run < most_runs { length } else { gx.abs() },
            )
        } else {
            let length = roll(rng, MIN_RUN, MAX_RUN).min(gy.abs());
            (
                0,
                gy.signum(),
                if run < most_runs { length } else { gy.abs() },
            )
        };
        for _ in 0..length {
            let (nx, ny) = (x + dx, y + dy);
            if nx < min_x || nx > max_x || ny < min_y || ny > max_y {
                break;
            }
            (x, y) = (nx, ny);
            dig(level, x, y);
        }
    }
}

/// The reached cell nearest to `from` (straight-line), first in row order on
/// ties.
pub(super) fn nearest_reached(level: &Level, reach: &Reach, from: (i64, i64)) -> (i64, i64) {
    let (min_x, min_y, width, height) = level.bounds();
    let mut best: Option<(i64, (i64, i64))> = None;
    for y in min_y..min_y + height {
        for x in min_x..min_x + width {
            if reach.is(x, y) {
                let d = (x - from.0).pow(2) + (y - from.1).pow(2);
                if best.is_none_or(|(best_d, _)| d < best_d) {
                    best = Some((d, (x, y)));
                }
            }
        }
    }
    best.expect("something reached").1
}
