//! What a waist piece rolls about itself, with where its edges are on the body.

use serde::{Deserialize, Serialize};

use crate::body::Frame;

/// The item kind a waist piece was made for, which says where it is worn.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum WaistKind {
    /// Outer on Waist: everything but the one below.
    Armour,
    /// Laminated plate tassets wrapping the thighs down to the knee, with knee cops:
    /// Outer on Waist and Legs (agreed with the legs' pieces, the owner's decision).
    Cuirassier,
}

/// What shape of piece it is, rolled after the construction.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum WaistPiece {
    /// A broad armoured band round the waist, maybe with a plate over the belly.
    Girdle,
    /// Hoops round the front of the hips, maybe with a culet behind.
    Fauld,
    /// A fauld with plates hanging over the thighs.
    Tassets,
    /// Separate panels hung all round: kusazuri, steppe flaps.
    Panels,
    /// Narrow strips in rows.
    Pteruges,
    /// One skirt, closed or split: mail, scale, lamellar, a tonlet.
    Skirt,
    /// Mail shorts over the hips and groin.
    Breeches,
}

impl WaistPiece {
    pub const ALL: [Self; 7] = [
        Self::Girdle,
        Self::Fauld,
        Self::Tassets,
        Self::Panels,
        Self::Pteruges,
        Self::Skirt,
        Self::Breeches,
    ];
}

/// How far down a piece hangs.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Length {
    /// A band at the waist only: girdles.
    Band,
    /// To the crotch.
    Hip,
    /// To the middle of the thigh.
    Thigh,
    /// To just above the knee.
    Knee,
}

impl Length {
    /// Where every waist piece's top is: 30 mm above the waist, tucked under the torso
    /// piece's hem (agreed with the torso's pieces).
    pub fn top(frame: &Frame) -> i32 {
        frame.waist + 30
    }

    /// Where its lower edge is at the front, in millimetres from the ground.
    pub fn bottom(self, frame: &Frame) -> i32 {
        match self {
            Self::Band => frame.waist - 90,
            Self::Hip => frame.hip - frame.thigh,
            Self::Thigh => frame.hip - (frame.hip - frame.knee) * 11 / 20,
            Self::Knee => frame.knee + 20,
        }
    }
}

/// Where a skirt is split, so its wearer can ride or stride.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Opening {
    Closed,
    /// Split up each side.
    Sides,
    /// Split up the front and back, for riding.
    FrontAndBack,
}
