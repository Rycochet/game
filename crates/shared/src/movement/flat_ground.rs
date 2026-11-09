//! Ground the wilderness keeps level: in a poi's cutout (its site's first
//! level set in), each group of cells that aren't out in the open (a
//! building, a castle's walls, a cave's rock, a pond) stands at one height,
//! while its outside cells (streets, yards, fields) follow the land
//! (`wilderness::Cutout::flats`, owner). Walking there costs and looks alike.

use serde::{Deserialize, Serialize};

/// A rectangle of wilderness cells whose covered cells (not
/// `Cell::is_outside`) are all at one height (`WildHeights::with_flats`).
/// Only its rectangle is told to the client, so a building's outline isn't
/// learnt before its cells.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct FlatGround {
    /// `(min_x, min_y, width, height)`, in cells.
    pub bounds: (i64, i64, i64, i64),
    /// Its height, in 16ths of an elevation (as `WildHeights::at`).
    pub height: i32,
}

impl FlatGround {
    /// Whether a cell is in its rectangle.
    pub fn contains(&self, x: i64, y: i64) -> bool {
        let (min_x, min_y, width, height) = self.bounds;
        (min_x..min_x + width).contains(&x) && (min_y..min_y + height).contains(&y)
    }
}
