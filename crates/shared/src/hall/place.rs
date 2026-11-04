//! Where a gate stands in the Hall, and its number along it.

use super::{GATE_SPACING, GateAddress, GateCode, GateDesign, Tile, WALKWAY_ROWS, tile, wrap_x};

/// Which side of the walkway a gate is on, facing out into the void.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Side {
    North,
    South,
}

/// One gate's place: its column (every `GATE_SPACING` tiles from `x = 0`)
/// and side. Gates are numbered along the Hall, north then south in each
/// column, so the number is `2 × column + side`: every `u32` is one gate.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct GatePlace {
    pub column: u32,
    pub side: Side,
}

impl GatePlace {
    /// The gate with this number.
    pub fn from_number(number: u32) -> Self {
        let side = if number & 1 == 0 {
            Side::North
        } else {
            Side::South
        };
        Self {
            column: number >> 1,
            side,
        }
    }

    /// Its number along the Hall.
    pub fn number(self) -> u32 {
        (self.column << 1) | (self.side == Side::South) as u32
    }

    /// The gate standing on this tile, if one does.
    pub fn at(x: i64, y: i64) -> Option<Self> {
        match tile(x, y) {
            Tile::Gate(side) => Some(Self {
                column: (wrap_x(x) / GATE_SPACING) as u32,
                side,
            }),
            _ => None,
        }
    }

    /// The tile it stands on.
    pub fn x(self) -> i64 {
        i64::from(self.column) * GATE_SPACING
    }

    pub fn y(self) -> i64 {
        match self.side {
            Side::North => -1,
            Side::South => WALKWAY_ROWS,
        }
    }

    /// The walkway tile in front of it.
    pub fn front(self) -> (i64, i64) {
        match self.side {
            Side::North => (self.x(), 0),
            Side::South => (self.x(), WALKWAY_ROWS - 1),
        }
    }

    /// The planet and World Gate it leads to.
    pub fn address(self) -> GateAddress {
        GateAddress::of(self)
    }

    /// What it is built like.
    pub fn design(self) -> GateDesign {
        GateDesign::of(self)
    }

    /// Its code for players.
    pub fn code(self) -> GateCode {
        GateCode::of(self)
    }
}
