//! Which way a place lies from a town: eight ways, or here.

use serde::{Deserialize, Serialize};

use super::offset;
use crate::lang::Selector;

/// One of the eight ways round the compass, or Here (the same spot).
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum Direction {
    North,
    NorthEast,
    East,
    SouthEast,
    South,
    SouthWest,
    West,
    NorthWest,
    Here,
    // New values go at the end: messages carry one by its place.
}

/// tan 22.5° as a fraction (0.414213…), where one way gives on to the next.
const TAN_NUMERATOR: i128 = 408;
const TAN_DENOMINATOR: i128 = 985;

impl Direction {
    pub const ALL: [Self; 9] = [
        Self::North,
        Self::NorthEast,
        Self::East,
        Self::SouthEast,
        Self::South,
        Self::SouthWest,
        Self::West,
        Self::NorthWest,
        Self::Here,
    ];

    /// The way from world map position `from` to `to`, on a planet `width`
    /// points round (the near way round). Whole numbers only; a way exactly on
    /// the line between two goes to the north, east, south or west one.
    pub fn between(from: (i64, i64), to: (i64, i64), width: i64) -> Self {
        let (dx, dy) = offset(from, to, width);
        let (across, down) = (i128::from(dx).abs(), i128::from(dy).abs());
        if across == 0 && down == 0 {
            return Self::Here;
        }
        let east = dx > 0;
        let south = dy > 0;
        if down * TAN_DENOMINATOR <= across * TAN_NUMERATOR {
            if east { Self::East } else { Self::West }
        } else if across * TAN_DENOMINATOR <= down * TAN_NUMERATOR {
            if south { Self::South } else { Self::North }
        } else {
            match (east, south) {
                (true, false) => Self::NorthEast,
                (true, true) => Self::SouthEast,
                (false, true) => Self::SouthWest,
                (false, false) => Self::NorthWest,
            }
        }
    }
}

impl Selector for Direction {
    fn fact(&self) -> &'static str {
        "direction"
    }

    fn key(&self) -> &'static str {
        match self {
            Self::North => "north",
            Self::NorthEast => "north-east",
            Self::East => "east",
            Self::SouthEast => "south-east",
            Self::South => "south",
            Self::SouthWest => "south-west",
            Self::West => "west",
            Self::NorthWest => "north-west",
            Self::Here => "here",
        }
    }
}
