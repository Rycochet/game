//! What is at any tile of the Hall, and its colour's hue.

use super::{GATE_SPACING, HALL_LENGTH, WALKWAY_ROWS};
use crate::hall::Side;

/// What a tile of the Hall is.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Tile {
    /// The marble walkway.
    Walkway,
    /// The floor a gate stands on, just beyond the walkway's edge.
    Gate(Side),
    /// Nothing: the void, with the walkway's edge falling away into it.
    Void,
}

/// `x` brought into `0..HALL_LENGTH`: the same tile, the Hall having come
/// round.
pub fn wrap_x(x: i64) -> i64 {
    x.rem_euclid(HALL_LENGTH)
}

/// What is at `(x, y)`. Rows `0..WALKWAY_ROWS` are walkway; the rows just
/// beyond them hold a gate every `GATE_SPACING` tiles, and void between.
pub fn tile(x: i64, y: i64) -> Tile {
    if (0..WALKWAY_ROWS).contains(&y) {
        return Tile::Walkway;
    }
    let side = match y {
        -1 => Side::North,
        WALKWAY_ROWS => Side::South,
        _ => return Tile::Void,
    };
    if wrap_x(x) % GATE_SPACING == 0 {
        Tile::Gate(side)
    } else {
        Tile::Void
    }
}

/// The hue of the marble at `x`, in degrees (`0..360`): once round the colour
/// wheel over the whole Hall, so it comes round again with the Hall and shows
/// roughly where along it you are. Only for looks.
pub fn hue(x: i64) -> f64 {
    wrap_x(x) as f64 / HALL_LENGTH as f64 * 360.0
}
