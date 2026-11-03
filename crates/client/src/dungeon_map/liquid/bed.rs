//! How deep a liquid cell's bed is, and whether it is a narrow channel.

use shared::dungeon::Level;

use super::{FULL_DEPTH, HALF_CELL, NARROW_DEPTH, Outline, Point, around, wet};

/// The depth of a cell's bed at a point in a quarter.
#[derive(Clone, Copy)]
pub(super) enum Bed {
    Flat(f32),
    /// Half deep in the middle of the cell, full at the east-west edge.
    SlopeEast,
    /// Half deep in the middle of the cell, full at the north-south edge.
    SlopeSouth,
}

impl Bed {
    pub(super) fn depth(self, (east, south): Point) -> f32 {
        let slope = |along: f32| NARROW_DEPTH + (FULL_DEPTH - NARROW_DEPTH) * along / HALF_CELL;
        match self {
            Self::Flat(depth) => depth,
            Self::SlopeEast => slope(east),
            Self::SlopeSouth => slope(south),
        }
    }
}

/// Whether a liquid cell is narrow: floor on two opposite sides.
pub(super) fn is_narrow(level: &Level, x: i64, y: i64) -> bool {
    let wet_at = |x, y| wet(level.cell(x, y));
    (!wet_at(x, y - 1) && !wet_at(x, y + 1)) || (!wet_at(x + 1, y) && !wet_at(x - 1, y))
}

/// The bed of a quarter of the liquid cell `(x, y)`.
pub(super) fn bed_at(
    level: &Level,
    (x, y): (i64, i64),
    quarter: (f32, f32),
    outline: Outline,
) -> Bed {
    if !is_narrow(level, x, y) {
        return Bed::Flat(FULL_DEPTH);
    }
    let [beside, ahead, _] = around((x, y), quarter);
    // The next cell of the channel, if it is narrow too, keeps the same depth.
    let carries_on = |(x, y): (i64, i64)| level.cell(x, y).is_liquid() && is_narrow(level, x, y);
    match outline {
        Outline::AlongEast if !carries_on(beside) => Bed::SlopeEast,
        Outline::AlongSouth if !carries_on(ahead) => Bed::SlopeSouth,
        _ => Bed::Flat(NARROW_DEPTH),
    }
}
