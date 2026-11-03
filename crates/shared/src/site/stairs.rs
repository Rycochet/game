//! Stairs: up or down, where, and their styles.

use serde::{Deserialize, Serialize};

use super::Exit;
use crate::poi::PoiKind;

/// Which way a staircase goes, for its picture.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum StairsDirection {
    Up,
    Down,
}

/// What a way between two levels is built as. Only for looks: all are used
/// the same way.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum StairsStyle {
    /// Winding round a central column.
    Spiral,
    /// Two flights side by side, turning back half way up.
    Turning,
    /// A wooden ladder leaning from the middle of the bottom.
    Ladder,
    /// A thick pole, leaning like a ladder, with rungs out of alternate sides.
    LogLadder,
    /// Hanging straight down, with sagging rope for rungs.
    RopeLadder,
    /// Hanging straight down, with wooden slats for rungs.
    SlatLadder,
    /// A single hanging rope with knots.
    KnottedRope,
}

impl StairsStyle {
    pub const ALL: [Self; 7] = [
        Self::Spiral,
        Self::Turning,
        Self::Ladder,
        Self::LogLadder,
        Self::RopeLadder,
        Self::SlatLadder,
        Self::KnottedRope,
    ];

    /// The next one, going round `ALL` (dev tools, to show them all).
    pub fn next(self) -> Self {
        let index = Self::ALL.iter().position(|&style| style == self);
        Self::ALL[(index.unwrap_or(0) + 1) % Self::ALL.len()]
    }

    /// The styles that suit a kind of place.
    pub(super) fn choices(kind: PoiKind) -> &'static [Self] {
        use PoiKind::*;
        use StairsStyle::*;
        match kind {
            Castle | Fort | StoneTower | StoneCottage | Warehouse | ArcheryRange | AdobeHouse
            | AdobeStore | StonePyramid | SandPyramid => &[Spiral, Turning],
            StiltHut | GreenStiltHut => &[Ladder, LogLadder],
            ThatchedHut | RedTent | BlueTent | GreenTent | RedRoofCottage | GreyRoofCottage
            | SnowyCottage | WoodenTower => &[Turning, Ladder, LogLadder],
            Crypt | MossyCrypt | BloodyCrypt | Mausoleum | Grave | Ruins | Barrel | Barrels => {
                &[Spiral, Turning, Ladder]
            }
            GrassyCave | EarthCave | RockCave | Boulders | IceCave | IceShelter => {
                &[Ladder, LogLadder, RopeLadder, SlatLadder, KnottedRope]
            }
            Farm | HerbalistHut => &[Turning, Ladder],
            // Towns have no stairs yet.
            Hamlet | Village | Town | City => &[Turning],
        }
    }
}

/// A staircase on a walkable cell. Using it takes you to `to`, where you
/// arrive on the matching staircase.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Stairs {
    pub x: i64,
    pub y: i64,
    pub direction: StairsDirection,
    pub to: Exit,
}
