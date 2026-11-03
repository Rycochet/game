//! The thin bars a wall cell is made of, and lines hitting them.

use super::{CELL_UNITS, WALL_HALF_UNITS, WallShape};

/// A bar of wall: its smallest and largest corners, in units.
pub type WallBar = ((i64, i64), (i64, i64));

/// The two bars of a wall of this shape in `cell`.
pub(super) fn bars_of(cell: (i64, i64), shape: WallShape) -> [WallBar; 2] {
    let (cx, cy) = (cell.0 * CELL_UNITS, cell.1 * CELL_UNITS);
    let [north, east, south, west] = shape.reach();
    let t = WALL_HALF_UNITS;
    [
        ((cx - west, cy - t), (cx + east, cy + t)),
        ((cx - t, cy - north), (cx + t, cy + south)),
    ]
}

/// Whether a line touches a box given by its smallest and largest corners:
/// their ranges overlap, and the box's corners aren't all on one side of it.
pub(super) fn line_hits_box(
    from: (i64, i64),
    to: (i64, i64),
    low: (i64, i64),
    high: (i64, i64),
) -> bool {
    if from.0.max(to.0) < low.0
        || from.0.min(to.0) > high.0
        || from.1.max(to.1) < low.1
        || from.1.min(to.1) > high.1
    {
        return false;
    }
    let side = |x: i64, y: i64| (to.0 - from.0) * (y - from.1) - (to.1 - from.1) * (x - from.0);
    let sides = [
        side(low.0, low.1),
        side(high.0, low.1),
        side(low.0, high.1),
        side(high.0, high.1),
    ];
    !(sides.iter().all(|&s| s > 0) || sides.iter().all(|&s| s < 0))
}
