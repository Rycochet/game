//! Open water in the wilderness, and which of it is shallow enough to wade.

use super::SHALLOWS_REACH;
use super::samples::Samples;

/// Which cells round a rectangle are open water (unfrozen sea for now).
pub(super) struct Water {
    min_x: i64,
    min_y: i64,
    width: i64,
    wet: Vec<bool>,
}

impl Water {
    /// The water of `(min_x, min_y, width, height)` and `SHALLOWS_REACH` round
    /// it, from samples covering that much (see `Samples::new`).
    pub fn new(
        samples: &Samples,
        low_water: f64,
        (min_x, min_y, width, height): (i64, i64, i64, i64),
    ) -> Self {
        let (min_x, min_y) = (min_x - SHALLOWS_REACH, min_y - SHALLOWS_REACH);
        let (width, height) = (width + 2 * SHALLOWS_REACH, height + 2 * SHALLOWS_REACH);
        let mut wet = Vec::with_capacity((width * height) as usize);
        for y in min_y..min_y + height {
            for x in min_x..min_x + width {
                wet.push(samples.height(x, y) < low_water && !samples.frozen(x, y));
            }
        }
        Self {
            min_x,
            min_y,
            width,
            wet,
        }
    }

    /// Whether a cell is open water.
    pub fn is_wet(&self, x: i64, y: i64) -> bool {
        self.wet[((y - self.min_y) * self.width + x - self.min_x) as usize]
    }

    /// Whether a wet cell has land within `SHALLOWS_REACH`, straight-line
    /// between middles: shallow enough to wade.
    pub fn is_shallow(&self, x: i64, y: i64) -> bool {
        let reach = SHALLOWS_REACH;
        (-reach..=reach).any(|dy| {
            (-reach..=reach)
                .any(|dx| dx * dx + dy * dy <= reach * reach && !self.is_wet(x + dx, y + dy))
        })
    }
}
