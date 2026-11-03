//! What a piece for the hands is made of: `HandsPiece`, `Fingers`, `Cuff`, `Knuckles`.

use serde::{Deserialize, Serialize};

use crate::body::Frame;

/// What kind of hand piece an item is, which says the layer and places it takes and so
/// what it may be rolled as.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum HandsPiece {
    /// Outer, over the hands and fingers.
    Gauntlets,
    /// Outer, over the back of the hand and wrist only, so rings stay free.
    Demi,
    /// Middle, under gauntlets.
    Padded,
}

/// How the fingers are covered.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Fingers {
    /// Each finger on its own.
    Fingered,
    /// All but the thumb together, under one set of plates or one bag of mail.
    Mitten,
    /// Not at all: the back of the hand and the wrist only.
    Open,
}

/// The part over the wrist and up the forearm, over the end of any vambrace.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Cuff {
    /// A band round the wrist.
    Wrist,
    /// One plate narrowing at the wrist and flaring out: 14th century.
    Hourglass,
    /// Flared like a bell.
    Bell,
    /// A bell drawn out to a point on the outside of the forearm.
    Pointed,
    /// To the middle of the forearm.
    Long,
}

impl Cuff {
    /// How far up the forearm it reaches from the wrist, in millimetres on this frame
    /// (agreed with the arms' pieces, which it goes over).
    pub fn reach(self, frame: &Frame) -> i32 {
        let human = match self {
            Self::Wrist => 50,
            Self::Hourglass => 100,
            Self::Bell => 110,
            Self::Pointed => 140,
            Self::Long => 200,
        };
        human * frame.wrist_length / 260
    }

    /// How much wider than the wrist its open end is, out of 100.
    pub fn flare(self) -> i32 {
        match self {
            Self::Wrist => 0,
            Self::Hourglass => 45,
            Self::Bell | Self::Pointed => 55,
            Self::Long => 30,
        }
    }
}

/// What is over the knuckles.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Knuckles {
    Plain,
    /// A raised bar across them.
    Ridged,
    /// A spike on each one (gadlings), or studs on leather.
    Spiked,
    /// A disc on the back of the hand, as on a jousting manifer.
    Rondel,
}
