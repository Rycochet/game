//! Fencing a field, with gates and gaps.

use super::super::grid::{chance, roll};
use super::{FENCE_GAP, FENCE_GAP_PERCENT, Plot};
use crate::dungeon::{Cell, Level};
use crate::rng::Rng;

/// The cells round a plot's edge, going round it from its north-west corner.
pub(super) fn fence_line(plot: Plot) -> Vec<(i64, i64)> {
    let Plot { x0, y0, x1, y1 } = plot;
    let top = (x0..x1 - 1).map(|x| (x, y0));
    let right = (y0..y1 - 1).map(|y| (x1 - 1, y));
    let bottom = (x0 + 1..x1).rev().map(|x| (x, y1 - 1));
    let left = (y0 + 1..y1).rev().map(|y| (x0, y));
    top.chain(right).chain(bottom).chain(left).collect()
}

/// Sows a plot and fences it, with its gate on the part of the fence (not a
/// corner) nearest `door`. Gives the cell outside the gate.
pub(super) fn fence_field(
    level: &mut Level,
    rng: &mut Rng,
    plot: Plot,
    door: (i64, i64),
    ruin: u8,
) -> (i64, i64) {
    let Plot { x0, y0, x1, y1 } = plot;
    for y in y0 + 1..y1 - 1 {
        for x in x0 + 1..x1 - 1 {
            level.set(x, y, Cell::Field);
        }
    }
    let line = fence_line(plot);
    let corner = |&(x, y): &(i64, i64)| (x == x0 || x == x1 - 1) && (y == y0 || y == y1 - 1);
    let distance = |&(x, y): &(i64, i64)| (x - door.0).pow(2) + (y - door.1).pow(2);
    let gate = line
        .iter()
        .filter(|spot| !corner(spot))
        .min_by_key(|spot| distance(spot))
        .copied()
        .expect("a fence has sides");
    let mut gap = 0;
    for &(x, y) in &line {
        if ruin > 0 && gap == 0 && chance(rng, FENCE_GAP_PERCENT * u32::from(ruin)) {
            gap = roll(rng, FENCE_GAP.0, FENCE_GAP.1);
        }
        let cell = if (x, y) == gate {
            Cell::FieldGate
        } else if gap > 0 {
            Cell::Ground
        } else {
            Cell::Fence
        };
        level.set(x, y, cell);
        gap = (gap - 1).max(0);
    }
    let out = if gate.1 == y0 {
        (0, -1)
    } else if gate.0 == x1 - 1 {
        (1, 0)
    } else if gate.1 == y1 - 1 {
        (0, 1)
    } else {
        (-1, 0)
    };
    (gate.0 + out.0, gate.1 + out.1)
}
