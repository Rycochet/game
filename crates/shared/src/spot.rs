//! Positions on a level in whole units, 240 to a cell (`relief::UNITS_PER_CELL`),
//! and distances between them by whole-number square roots. See
//! `docs/context/shared/clearance.md`.
//!
//! Cell `(x, y)`'s middle is the spot `(x × 240, y × 240)`, as in `relief`, so a
//! cell covers 120 units either side of it. Spots on the Hall's levels use the
//! Hall's own unwrapped `x`, as its cells do (`hall::wrap_x` brings a cell round).

use serde::{Deserialize, Serialize};

use crate::relief::UNITS_PER_CELL;

/// Units across a cell.
pub const CELL_SPAN: i64 = UNITS_PER_CELL as i64;

/// Units from a cell's middle to its edge.
pub const HALF_SPAN: i64 = CELL_SPAN / 2;

/// A point on a level, in units: `x` east, `y` south.
#[derive(
    Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize,
)]
pub struct Spot {
    pub x: i64,
    pub y: i64,
}

impl Spot {
    pub const fn new(x: i64, y: i64) -> Self {
        Self { x, y }
    }

    /// The middle of cell `(x, y)`.
    pub const fn middle_of((x, y): (i64, i64)) -> Self {
        Self::new(x * CELL_SPAN, y * CELL_SPAN)
    }

    /// The cell this is in. A spot on the line between two cells is in the one
    /// east (or south) of it.
    pub fn cell(self) -> (i64, i64) {
        (
            (self.x + HALF_SPAN).div_euclid(CELL_SPAN),
            (self.y + HALF_SPAN).div_euclid(CELL_SPAN),
        )
    }

    /// This spot moved by `(dx, dy)` units.
    pub const fn offset(self, dx: i64, dy: i64) -> Self {
        Self::new(self.x + dx, self.y + dy)
    }

    /// The square of the distance to `other`, exactly.
    pub fn distance_squared(self, other: Self) -> u128 {
        let (dx, dy) = ((other.x - self.x) as i128, (other.y - self.y) as i128);
        (dx * dx + dy * dy) as u128
    }

    /// The distance to `other`, rounded down to a whole unit.
    pub fn distance(self, other: Self) -> u64 {
        root(self.distance_squared(other))
    }

    /// The spot `numerator / denominator` of the way to `other` (rounded
    /// towards `self`'s side by truncating).
    pub fn towards(self, other: Self, numerator: i64, denominator: i64) -> Self {
        let part =
            |a: i64, b: i64| a + ((b - a) as i128 * numerator as i128 / denominator as i128) as i64;
        Self::new(part(self.x, other.x), part(self.y, other.y))
    }
}

/// The whole-number square root of `n`, rounded down: the same everywhere,
/// with no floats.
pub fn root(n: u128) -> u64 {
    n.isqrt() as u64
}

#[cfg(test)]
mod tests;
