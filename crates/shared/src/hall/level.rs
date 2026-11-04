//! A stretch of the Hall as dungeon cells, for the rules both sides check
//! steps by and for drawing. The Hall is far too long to hold whole, so each
//! side makes the stretch round the player it needs.

use super::{Tile, WALKWAY_ROWS, tile};
use crate::dungeon::{Cell, Level};

/// The first row of a stretch: one more row of void beyond each gate's
/// floor, so the drawn edge has void round it.
pub const LEVEL_TOP: i64 = -2;
/// Rows in a stretch: the walkway, the gates' rows and a row of void beyond
/// each.
pub const LEVEL_ROWS: i64 = WALKWAY_ROWS + 4;

/// The cell for a tile: walkway is floor, a gate's floor a door for now, the
/// void a hole.
pub fn cell(x: i64, y: i64) -> Cell {
    match tile(x, y) {
        Tile::Walkway => Cell::Floor,
        Tile::Gate(_) => Cell::Door,
        Tile::Void => Cell::Hole,
    }
}

/// The Hall from `min_x`, `width` tiles east, as a level. Its `x` are the
/// Hall's own, not wrapped, so a stretch over the seam is all one.
pub fn level(min_x: i64, width: i64) -> Level {
    let mut level = Level::new(min_x, LEVEL_TOP, width, LEVEL_ROWS);
    for y in LEVEL_TOP..LEVEL_TOP + LEVEL_ROWS {
        for x in min_x..min_x + width {
            level.set(x, y, cell(x, y));
        }
    }
    level
}
