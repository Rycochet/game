//! Distances the searches go by.

use crate::movement::{DIAGONAL_COST, STRAIGHT_COST};

/// The least a route between two points can cost: what walking it would, with
/// nothing in the way. Never more than the real cost, which A* relies on to
/// find the quickest route. (If terrain faster than floor is ever added, this
/// must assume it.)
pub(super) fn estimate(from: (i64, i64), to: (i64, i64)) -> u32 {
    let (dx, dy) = ((to.0 - from.0).abs(), (to.1 - from.1).abs());
    let (long, short) = (dx.max(dy) as u32, dx.min(dy) as u32);
    (long - short) * STRAIGHT_COST + short * DIAGONAL_COST
}

pub(super) fn distance_squared(a: (i64, i64), b: (i64, i64)) -> i64 {
    (a.0 - b.0).pow(2) + (a.1 - b.1).pow(2)
}
