//! How high the wilderness's cells count as for walking, for the level the
//! player is on (`shared::movement::WildHeights`).

use bevy::prelude::*;
use shared::movement::WildHeights;

/// The heights of the wilderness level's cells, worked out by `player` from the
/// world map's ground whenever the level is the wilderness and its window
/// moves; `None` anywhere else. Step costs and routes there go by them, as the
/// server's do.
#[derive(Resource, Default)]
pub struct WildCellHeights {
    /// The level's bounds they were worked out for.
    pub bounds: (i64, i64, i64, i64),
    pub heights: Option<WildHeights>,
}

impl WildCellHeights {
    /// The heights at each end of a step, if both are known.
    pub fn ends(&self, from: (i64, i64), to: (i64, i64)) -> Option<(i32, i32)> {
        let heights = self.heights.as_ref()?;
        (heights.covers(from.0, from.1) && heights.covers(to.0, to.1))
            .then(|| (heights.at(from.0, from.1), heights.at(to.0, to.1)))
    }
}
