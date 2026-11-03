//! What an arm piece rolls about itself, with where its parts are along the arm.

use serde::{Deserialize, Serialize};

use crate::body::{Frame, Side};

/// How much of the arm a piece covers. Every reach takes the Arms place.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Reach {
    /// Part of the forearm above the wrist: bracers.
    Wrist,
    /// The forearm, wrist to below the elbow.
    Forearm,
    /// The elbow alone: couters.
    Elbow,
    /// The forearm and the elbow.
    ForearmAndElbow,
    /// The upper arm and the elbow: rerebraces.
    UpperAndElbow,
    /// Shoulder to wrist.
    Whole,
}

impl Reach {
    pub const ALL: [Self; 6] = [
        Self::Wrist,
        Self::Forearm,
        Self::Elbow,
        Self::ForearmAndElbow,
        Self::UpperAndElbow,
        Self::Whole,
    ];

    pub fn forearm(self) -> bool {
        matches!(self, Self::Forearm | Self::ForearmAndElbow | Self::Whole)
    }

    pub fn elbow(self) -> bool {
        matches!(
            self,
            Self::Elbow | Self::ForearmAndElbow | Self::UpperAndElbow | Self::Whole
        )
    }

    pub fn upper(self) -> bool {
        matches!(self, Self::UpperAndElbow | Self::Whole)
    }

    /// From and to how far down the arm it reaches, in millimetres below the shoulder
    /// joint (agreed with the shoulders' and hands' pieces: a rerebrace starts 100 mm
    /// down, under the pauldron's lames; everything stops 10 mm above the wrist).
    pub fn span(self, frame: &Frame) -> (i32, i32) {
        let elbow = frame.elbow_length;
        let wrist = elbow + frame.wrist_length - 10;
        match self {
            Self::Wrist => (wrist - 150, wrist),
            Self::Forearm => (elbow + 40, wrist),
            Self::Elbow => (elbow - 60, elbow + 60),
            Self::ForearmAndElbow => (elbow - 60, wrist),
            Self::UpperAndElbow => (100, elbow + 60),
            Self::Whole => (100, wrist),
        }
    }
}

/// Which arms a piece is on.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Sides {
    Both,
    Left,
    Right,
}

impl Sides {
    /// The arms it is on.
    pub fn sides(self) -> &'static [Side] {
        match self {
            Self::Both => &Side::BOTH,
            Self::Left => &[Side::Left],
            Self::Right => &[Side::Right],
        }
    }

    pub fn count(self) -> u32 {
        self.sides().len() as u32
    }
}

/// The elbow's cup on rigid pieces.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Couter {
    /// No elbow piece: the reach misses the elbow, or it isn't rigid there.
    None,
    /// A plain cup over the point of the elbow.
    Cup,
    /// A cup with a fan on the outside; the style draws the fan.
    Winged,
}

/// Fantasy spikes, on plate and leather.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Spikes {
    None,
    /// One spike on the point of the couter.
    Elbow,
    /// A row down the outside of the forearm.
    Ridge,
}
