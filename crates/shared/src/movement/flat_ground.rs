//! Ground the wilderness keeps level: a poi's cutout (its site's first level
//! set in), so its buildings and fields never stand on a slope. Walking there
//! costs and looks flat alike.

use serde::{Deserialize, Serialize};

use super::{WildHeights, WorldGround};

/// A rectangle of wilderness cells all at one height.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct FlatGround {
    /// `(min_x, min_y, width, height)`, in cells.
    pub bounds: (i64, i64, i64, i64),
    /// Its height, in 16ths of an elevation (as `WildHeights::at`).
    pub height: i32,
}

impl FlatGround {
    /// A cutout's rectangle, levelled at the height its middle cell would
    /// have had.
    pub fn of_cutout(ground: &WorldGround, bounds: (i64, i64, i64, i64)) -> Self {
        let (min_x, min_y, width, height) = bounds;
        let middle = (min_x + width / 2, min_y + height / 2);
        let heights = WildHeights::new(ground, (middle.0, middle.1, 1, 1));
        Self {
            bounds,
            height: heights.at(middle.0, middle.1),
        }
    }

    /// Whether a cell is in it.
    pub fn contains(&self, x: i64, y: i64) -> bool {
        let (min_x, min_y, width, height) = self.bounds;
        (min_x..min_x + width).contains(&x) && (min_y..min_y + height).contains(&y)
    }
}
