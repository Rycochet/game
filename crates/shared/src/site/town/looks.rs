//! What a town building's walls and roof are made of.

use serde::{Deserialize, Serialize};

/// What a building's walls are made of.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum WallKind {
    Wattle,
    Timber,
    Plaster,
    Brick,
    Stone,
}

impl WallKind {
    pub const ALL: [Self; 5] = [
        Self::Wattle,
        Self::Timber,
        Self::Plaster,
        Self::Brick,
        Self::Stone,
    ];

    /// Its weight in the smallest and largest towns.
    pub(super) fn odds(self) -> (u64, u64) {
        match self {
            Self::Wattle => (40, 2),
            Self::Timber => (35, 18),
            Self::Plaster => (15, 20),
            Self::Brick => (7, 30),
            Self::Stone => (3, 30),
        }
    }
}

/// What a building's roof is made of.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum RoofKind {
    MudAndSticks,
    Thatch,
    WoodenShingles,
    ClayTiles,
    SlateTiles,
}

impl RoofKind {
    pub const ALL: [Self; 5] = [
        Self::MudAndSticks,
        Self::Thatch,
        Self::WoodenShingles,
        Self::ClayTiles,
        Self::SlateTiles,
    ];

    /// Its weight in the smallest and largest towns.
    pub(super) fn odds(self) -> (u64, u64) {
        match self {
            Self::MudAndSticks => (45, 1),
            Self::Thatch => (35, 9),
            Self::WoodenShingles => (15, 25),
            Self::ClayTiles => (4, 35),
            Self::SlateTiles => (1, 30),
        }
    }
}
