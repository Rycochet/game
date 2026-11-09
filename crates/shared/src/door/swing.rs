//! How a door hangs: closed, open to one side of its wall, or broken.

use serde::{Deserialize, Serialize};

/// A door's state. Open and broken doors say which side of their wall the
/// leaf lies on.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Swing {
    Closed,
    Open(Side),
    /// Open for good: it can't be closed again.
    Broken(Side),
}

impl Swing {
    /// Whether it is closed: nothing walks, sees or shines through.
    pub fn is_shut(self) -> bool {
        self == Self::Closed
    }
}

/// One side of a wall, by the compass. A door in a wall running east-west
/// opens to the north or south; one in a wall running north-south, to the
/// east or west.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Side {
    North,
    East,
    South,
    West,
}

impl Side {
    /// The side of a door at `door` (in a wall running east-west if
    /// `runs_east_west`) that `from` is on. Someone standing in line with the
    /// wall counts as south or east.
    pub fn of(runs_east_west: bool, door: (i64, i64), from: (i64, i64)) -> Self {
        if runs_east_west {
            if from.1 < door.1 {
                Self::North
            } else {
                Self::South
            }
        } else if from.0 < door.0 {
            Self::West
        } else {
            Self::East
        }
    }

    /// The other side.
    pub fn opposite(self) -> Self {
        match self {
            Self::North => Self::South,
            Self::East => Self::West,
            Self::South => Self::North,
            Self::West => Self::East,
        }
    }

    /// Whether a leaf on this side fits a door in a wall running east-west if
    /// `runs_east_west`.
    pub fn fits(self, runs_east_west: bool) -> bool {
        matches!(self, Self::North | Self::South) == runs_east_west
    }

    /// The cell next to `(x, y)` on this side.
    pub fn beside(self, (x, y): (i64, i64)) -> (i64, i64) {
        match self {
            Self::North => (x, y - 1),
            Self::East => (x + 1, y),
            Self::South => (x, y + 1),
            Self::West => (x - 1, y),
        }
    }
}
