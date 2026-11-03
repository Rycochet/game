//! The parts shoulder armour is described by: cut, wings, haute pieces, span, guards, lacing, capes and ailettes.

use serde::{Deserialize, Serialize};

/// How a pair of pauldrons matches.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Cut {
    Matched,
    /// The left larger, its wings nearly meeting in the middle (Milanese).
    GreatLeft,
    /// The right cut away under the arm for a lance rest.
    LanceCut,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Wings {
    Small,
    Large,
}

/// An upright flange on a pauldron guarding the neck.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum HautePiece {
    None,
    Low,
    High,
}

impl HautePiece {
    /// How far it stands up from the cop, in millimetres on an ordinary body.
    pub fn height(self) -> i32 {
        match self {
            Self::None => 0,
            Self::Low => 25,
            Self::High => 50,
        }
    }
}

/// How big a shoulder guard or an ailette is.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Span {
    Small,
    Medium,
    Large,
}

impl Span {
    /// A shoulder guard's width round the arm and height, in millimetres on an
    /// ordinary body.
    pub fn guard(self) -> (i32, i32) {
        match self {
            Self::Small => (200, 200),
            Self::Medium => (260, 280),
            Self::Large => (330, 360),
        }
    }

    /// An ailette's width and height.
    pub fn ailette(self) -> (i32, i32) {
        match self {
            Self::Small => (120, 150),
            Self::Medium => (140, 175),
            Self::Large => (160, 200),
        }
    }

    /// How many rows a guard of this size has, fewest and most.
    pub(super) fn rows(self) -> (u8, u8) {
        match self {
            Self::Small => (3, 5),
            Self::Medium => (5, 6),
            Self::Large => (6, 7),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum GuardShape {
    Straight,
    /// Narrower at the bottom, wrapping the arm (tsubo-sode).
    Tapered,
    /// Wider at the bottom (hiro-sode).
    Flared,
}

/// How close the lacing of lamellae is.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Lacing {
    /// Covering the plates (kebiki).
    Dense,
    /// In spaced pairs (sugake).
    Spaced,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum CapeCut {
    /// Round the neck and over both shoulders: a Roman doubling.
    Round,
    /// From the back over each shoulder, tied down on the chest: a linothorax's top.
    Yoke,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Length {
    /// To the shoulders' edge.
    Short,
    /// To the middle of the upper arm.
    Long,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum AiletteShape {
    Rectangle,
    Rounded,
    Shield,
    Lozenge,
}
