//! Whether a straight line between two points is clear of walls.

use super::{CELL_UNITS, HALF_CELL, Level, WallShape, bars_of, line_hits_box};

/// Whether the straight line between the middles of two cells misses every wall:
/// the rule sight goes by, for something else that travels the same way.
pub fn line_is_clear(level: &Level, from: (i64, i64), to: (i64, i64)) -> bool {
    clear_line(
        level,
        (from.0 * CELL_UNITS, from.1 * CELL_UNITS),
        (to.0 * CELL_UNITS, to.1 * CELL_UNITS),
    )
}

/// Whether the straight line between two points (in units) misses every wall.
/// Goes through the cells it crosses, column by column from `from`.
pub(super) fn clear_line(level: &Level, from: (i64, i64), to: (i64, i64)) -> bool {
    let (dx, dy) = (to.0 - from.0, to.1 - from.1);
    let (first, last) = (cell_of(from.0), cell_of(to.0));
    let step = if last >= first { 1 } else { -1 };
    let mut column = first;
    loop {
        // The part of the line in this column.
        let left = (column * CELL_UNITS - HALF_CELL).max(from.0.min(to.0));
        let right = (column * CELL_UNITS + HALF_CELL).min(from.0.max(to.0));
        let (y1, y2) = if dx == 0 {
            (from.1, to.1)
        } else {
            (
                from.1 + (left - from.0) * dy / dx,
                from.1 + (right - from.0) * dy / dx,
            )
        };
        // One unit more each way, as the divisions above round.
        let (top, bottom) = (cell_of(y1.min(y2) - 1), cell_of(y1.max(y2) + 1));
        for row in top..=bottom {
            if let Some(shape) = level.wall_shape(column, row)
                && line_hits_wall(from, to, (column, row), shape)
            {
                return false;
            }
        }
        if column == last {
            return true;
        }
        column += step;
    }
}

/// The cell a coordinate (in units) is in.
pub(super) fn cell_of(units: i64) -> i64 {
    (units + HALF_CELL).div_euclid(CELL_UNITS)
}

/// Whether a line touches the wall in `cell`. The wall is two bars crossing
/// at the cell's centre.
pub(super) fn line_hits_wall(
    from: (i64, i64),
    to: (i64, i64),
    cell: (i64, i64),
    shape: WallShape,
) -> bool {
    bars_of(cell, shape)
        .iter()
        .any(|&(low, high)| line_hits_box(from, to, low, high))
}
