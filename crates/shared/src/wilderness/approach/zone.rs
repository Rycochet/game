//! The rectangle round a cutout a road's bend stays in.

use super::Point;

/// The rectangle round a cutout a road's bend stays in.
pub(super) struct Zone {
    pub(super) min: Point,
    pub(super) max: Point,
}

impl Zone {
    pub(super) fn contains(&self, (x, y): Point) -> bool {
        x >= self.min.0 && x <= self.max.0 && y >= self.min.1 && y <= self.max.1
    }

    /// Where a stretch from outside to inside first comes into it, near
    /// enough (halving the stretch until a twentieth of a cell).
    pub(super) fn entered(&self, outside: Point, inside: Point) -> Point {
        let (mut low, mut high) = (0.0, 1.0);
        let at = |t: f64| {
            (
                outside.0 + (inside.0 - outside.0) * t,
                outside.1 + (inside.1 - outside.1) * t,
            )
        };
        for _ in 0..24 {
            let middle = (low + high) / 2.0;
            if self.contains(at(middle)) {
                high = middle;
            } else {
                low = middle;
            }
        }
        at(high)
    }
}
