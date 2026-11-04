//! What a leg piece rolls about itself, with where its parts are on the leg.

use serde::{Deserialize, Serialize};

use crate::body::Frame;

/// The item kind a leg piece was made for, which says where it is worn.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum LegsKind {
    /// Outer on Legs: everything but the two below.
    Armour,
    /// Mail chausses with feet: Outer on Legs and Feet.
    Chausses,
    /// Quilted leggings worn under the rest: Middle on Legs, in place of trousers.
    Padded,
}

/// Which parts of the leg a piece covers.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Cover {
    /// Poleyns and knee guards.
    Knee,
    /// Greaves and schynbalds, the knee to the ankle.
    Shin,
    /// Cuisses and thigh guards, the crotch to above the knee.
    Thigh,
    ThighAndKnee,
    KneeAndShin,
    /// The crotch to the ankle.
    Whole,
    /// Bronze ankle guards, round the ankle bones and up the heel.
    Ankle,
}

impl Cover {
    pub const ALL: [Self; 7] = [
        Self::Knee,
        Self::Shin,
        Self::Thigh,
        Self::ThighAndKnee,
        Self::KneeAndShin,
        Self::Whole,
        Self::Ankle,
    ];

    pub fn thigh(self) -> bool {
        matches!(self, Self::Thigh | Self::ThighAndKnee | Self::Whole)
    }

    pub fn knee(self) -> bool {
        matches!(
            self,
            Self::Knee | Self::ThighAndKnee | Self::KneeAndShin | Self::Whole
        )
    }

    pub fn shin(self) -> bool {
        matches!(self, Self::Shin | Self::KneeAndShin | Self::Whole)
    }

    /// From and to what height it reaches, in millimetres from the ground, at the
    /// front (agreed with the waist's and feet's pieces: cuisses start at the crotch,
    /// greaves come down to the ankle, over a sabaton's top).
    pub fn span(self, frame: &Frame) -> (i32, i32) {
        let crotch = frame.hip - frame.thigh;
        let (knee, ankle) = (frame.knee, frame.ankle);
        let top = match self {
            Self::Thigh | Self::ThighAndKnee | Self::Whole => crotch,
            Self::Knee | Self::KneeAndShin => knee + 70,
            Self::Shin => knee - 40,
            Self::Ankle => ankle + 50,
        };
        let bottom = match self {
            Self::Thigh => knee + 60,
            Self::Knee | Self::ThighAndKnee => knee - 70,
            Self::Shin | Self::KneeAndShin | Self::Whole => ankle,
            Self::Ankle => ankle - 40,
        };
        (top, bottom)
    }
}

/// What covers the knee, on pieces that reach it.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Knee {
    /// Nothing of its own: the cover misses the knee, or it is soft there.
    None,
    /// A cup or boss over the knee.
    Cop,
    /// A cup with a fan on the outside; the style draws the fan.
    Winged,
    /// A flat plate standing above the knee, tied on: Japanese suneate's tate-age.
    Tall,
}

/// Plates strapped over mail chausses, as in the 13th and 14th centuries.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Reinforce {
    None,
    Poleyns,
    PoleynsAndSchynbalds,
    /// Quilted (gamboised) thighs over the mail, with poleyns on them.
    QuiltedThighs,
}

impl Reinforce {
    pub fn poleyns(self) -> bool {
        self != Self::None
    }
}

/// Fantasy ornaments, rare, on plate and leather.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Ornament {
    None,
    /// A spike on each knee cop.
    KneeSpikes,
    /// A blade-like ridge down each shin.
    ShinFins,
}
