//! Joining corridors, and doors between rooms, some of them secret.

use super::super::grid::{self, Reach};
use super::{Corridor, Shape, Spot, between};
use crate::dungeon::{Cell, Level};
use crate::rng::Rng;

/// Where a corridor ends at the wall of another corridor (made by an earlier
/// split), opens the wall between them.
pub(super) fn join_corridors(level: &mut Level, shape: &Shape, corridors: &[Corridor]) {
    let is_corridor = |level: &Level, x: i64, y: i64| {
        level.cell(x, y) == Cell::Floor && level.room(x, y).is_none()
    };
    for corridor in corridors {
        let f = corridor.floor;
        // Each end: the wall just past it, and the cell beyond that.
        let ends: Vec<Vec<(Spot, Spot, Spot)>> = if corridor.north_south {
            vec![
                (f.x0..f.x1)
                    .map(|x| ((x, f.y0), (x, f.y0 - 1), (x, f.y0 - 2)))
                    .collect(),
                (f.x0..f.x1)
                    .map(|x| ((x, f.y1 - 1), (x, f.y1), (x, f.y1 + 1)))
                    .collect(),
            ]
        } else {
            vec![
                (f.y0..f.y1)
                    .map(|y| ((f.x0, y), (f.x0 - 1, y), (f.x0 - 2, y)))
                    .collect(),
                (f.y0..f.y1)
                    .map(|y| ((f.x1 - 1, y), (f.x1, y), (f.x1 + 1, y)))
                    .collect(),
            ]
        };
        for end in ends {
            for (inside, wall, beyond) in end {
                if is_corridor(level, inside.0, inside.1)
                    && level.cell(wall.0, wall.1) == Cell::Wall
                    && shape.is_interior(wall.0, wall.1)
                    && is_corridor(level, beyond.0, beyond.1)
                {
                    level.set(wall.0, wall.1, Cell::Floor);
                }
            }
        }
    }
}

/// Adds doors in inside walls until every room and corridor can be reached
/// from `start`, preferring doors from corridors.
pub(super) fn add_doors(level: &mut Level, shape: &Shape, rng: &mut Rng, start: (i64, i64)) {
    let mut reach = Reach::new(level);
    reach.spread(level, &[start]);
    while reach.first_unreached(level).is_some() {
        let mut found: Option<(i64, i64)> = None;
        // Strict first (no door next to another), then relaxed.
        for strict in [true, false] {
            let mut candidates = Vec::new();
            let mut from_corridors = Vec::new();
            for y in shape.y..shape.y + shape.height {
                for x in shape.x..shape.x + shape.width {
                    if level.cell(x, y) != Cell::Wall || !shape.is_interior(x, y) {
                        continue;
                    }
                    if strict && grid::door_nearby(level, x, y) {
                        continue;
                    }
                    let Some((a, b)) = between(level, x, y) else {
                        continue;
                    };
                    let reached = match (reach.is(a.0, a.1), reach.is(b.0, b.1)) {
                        (true, false) => a,
                        (false, true) => b,
                        _ => continue,
                    };
                    candidates.push((x, y));
                    if level.room(reached.0, reached.1).is_none() {
                        from_corridors.push((x, y));
                    }
                }
            }
            let pool = if from_corridors.is_empty() {
                candidates
            } else {
                from_corridors
            };
            if !pool.is_empty() {
                found = Some(pool[rng.range_u64(0, pool.len() as u64) as usize]);
                break;
            }
        }
        match found {
            Some((x, y)) => {
                level.set(x, y, Cell::Door);
                reach.spread(level, &[(x, y)]);
            }
            None => {
                // Separated by more than one wall (it shouldn't happen): dig.
                if !grid::dig_to_unreached(level, &reach, |x, y| shape.is_interior(x, y)) {
                    return;
                }
                reach = Reach::new(level);
                reach.spread(level, &[start]);
            }
        }
    }
}

/// A few more doors between different rooms, making loops (and places for
/// secret doors).
pub(super) fn add_extra_doors(level: &mut Level, shape: &Shape, rng: &mut Rng, count: u16) {
    let mut candidates = Vec::new();
    for y in shape.y..shape.y + shape.height {
        for x in shape.x..shape.x + shape.width {
            if level.cell(x, y) != Cell::Wall || !shape.is_interior(x, y) {
                continue;
            }
            if let Some((a, b)) = between(level, x, y)
                && level.room(a.0, a.1) != level.room(b.0, b.1)
            {
                candidates.push((x, y));
            }
        }
    }
    for _ in 0..count {
        if candidates.is_empty() {
            return;
        }
        let (x, y) = candidates.swap_remove(rng.range_u64(0, candidates.len() as u64) as usize);
        if !grid::door_nearby(level, x, y) {
            level.set(x, y, Cell::Door);
        }
    }
}
