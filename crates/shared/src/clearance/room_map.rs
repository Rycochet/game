//! Where creatures of one size have room to stand, cell by cell.

use super::check_segment_by;
use crate::creature::Size;
use crate::dungeon::Level;
use crate::spot::Spot;

/// For each cell of a level, whether a creature of one size can stand at its
/// middle, and how squeezed it is there: what routes for that size search
/// over (`pathfinding::find_sized_path`). Made once per level and size, and
/// again when its doors change.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RoomMap {
    size: Size,
    /// Whether it is for a good swimmer, who can stand in water.
    swims: bool,
    bounds: (i64, i64, i64, i64),
    /// Thousandths slower (`squeeze_per_mille`), 0 for no room, row by row.
    squeeze: Vec<u32>,
}

impl RoomMap {
    pub fn new(level: &Level, size: Size) -> Self {
        Self::new_by(level, size, false)
    }

    /// The same for a good swimmer (`swims`, `Gait::swims_deep`), who can
    /// stand in water too (`has_footing_by`).
    pub fn new_by(level: &Level, size: Size, swims: bool) -> Self {
        let bounds = level.bounds();
        let (min_x, min_y, width, height) = bounds;
        let mut squeeze = Vec::with_capacity((width * height) as usize);
        for y in min_y..min_y + height {
            for x in min_x..min_x + width {
                let middle = Spot::middle_of((x, y));
                let here = check_segment_by(level, size, swims, middle, middle);
                squeeze.push(here.map_or(0, |passage| passage.squeeze_per_mille));
            }
        }
        Self {
            size,
            swims,
            bounds,
            squeeze,
        }
    }

    /// The size it is for.
    pub fn size(&self) -> Size {
        self.size
    }

    /// Whether it is for a good swimmer.
    pub fn swims(&self) -> bool {
        self.swims
    }

    /// How squeezed a creature standing at cell `(x, y)`'s middle is, in
    /// thousandths (1000: not at all; at most `MOST_SQUEEZE_PER_MILLE`), or
    /// `None` if it has no room there (or the cell is off the level).
    pub fn squeeze_at(&self, x: i64, y: i64) -> Option<u32> {
        let (min_x, min_y, width, height) = self.bounds;
        let (col, row) = (x - min_x, y - min_y);
        if !(0..width).contains(&col) || !(0..height).contains(&row) {
            return None;
        }
        Some(self.squeeze[(row * width + col) as usize]).filter(|&s| s > 0)
    }

    /// Whether a creature can stand at cell `(x, y)`'s middle.
    pub fn has_room(&self, x: i64, y: i64) -> bool {
        self.squeeze_at(x, y).is_some()
    }
}
