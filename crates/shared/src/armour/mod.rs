//! Armour: what every piece and set shares. Pieces for each body location are added by
//! their own work, built on `body::Frame`. See `docs/context/shared/armour.md`.

pub mod hands;

use serde::{Deserialize, Serialize};

use crate::rng::Rng;

/// How a piece of armour is made, decided first: it says what it looks like and what
/// it can be made of. A set has one for all its pieces.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Construction {
    /// Quilted layers of cloth, stuffed.
    Padded,
    /// Hardened or thick leather.
    Leather,
    /// Interlinked rings.
    Mail,
    /// Small plates on a backing, overlapping downwards.
    Scale,
    /// Small plates laced to each other, with no backing.
    Lamellar,
    /// Strips riveted across leathers, as on a lorica segmentata.
    Laminar,
    /// Plates riveted inside or under cloth: brigandines, coats of plates.
    PlatesInCloth,
    /// Plates joined by mail.
    MailAndPlate,
    /// Long strips down the limbs over a backing.
    Splint,
    /// Shaped solid plates, joined to move.
    Plate,
    /// Large polished plates over mail.
    Mirror,
}

impl Construction {
    pub const ALL: [Self; 11] = [
        Self::Padded,
        Self::Leather,
        Self::Mail,
        Self::Scale,
        Self::Lamellar,
        Self::Laminar,
        Self::PlatesInCloth,
        Self::MailAndPlate,
        Self::Splint,
        Self::Plate,
        Self::Mirror,
    ];

    /// Whether it bends with the body rather than holding its shape.
    pub fn flexible(self) -> bool {
        matches!(
            self,
            Self::Padded | Self::Mail | Self::Scale | Self::Lamellar
        )
    }
}

/// How a piece looks, shared by every piece of a set so they match: the owner's list of
/// harness styles, historical and fantasy. Mostly for plate; on other constructions it
/// changes only trim and edges.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Style {
    /// Plain polished plates, rolled edges.
    #[default]
    White,
    /// Pointed and ridged, flutes fanning out, cusped edges: 15th-century German.
    Gothic,
    /// Smooth and rounded, broad plates, roped edges: 15th-century Italian.
    Milanese,
    /// Parallel flutes over every plate: early 16th century.
    Maximilian,
    /// Etched bands with gilt lines: 16th-century English.
    Greenwich,
    /// Boxy plates, long cuffs with square cuts: early 15th century.
    Kastenbrust,
    /// Slender, leaf-pointed, no rivets showing.
    Elven,
    /// Thick and square, stepped edges, heavy bars.
    Dwarven,
}

impl Style {
    pub const ALL: [Self; 8] = [
        Self::White,
        Self::Gothic,
        Self::Milanese,
        Self::Maximilian,
        Self::Greenwich,
        Self::Kastenbrust,
        Self::Elven,
        Self::Dwarven,
    ];

    pub fn name(self) -> &'static str {
        match self {
            Self::White => "White",
            Self::Gothic => "Gothic",
            Self::Milanese => "Milanese",
            Self::Maximilian => "Maximilian",
            Self::Greenwich => "Greenwich",
            Self::Kastenbrust => "Kastenbrust",
            Self::Elven => "Elven",
            Self::Dwarven => "Dwarven",
        }
    }

    /// A style for a piece made on its own, plain the most often.
    pub fn roll(rng: &mut Rng) -> Self {
        rng.pick_weighted(&[
            (Self::White, 30),
            (Self::Gothic, 15),
            (Self::Milanese, 15),
            (Self::Maximilian, 10),
            (Self::Greenwich, 8),
            (Self::Kastenbrust, 5),
            (Self::Elven, 8),
            (Self::Dwarven, 8),
        ])
    }
}
