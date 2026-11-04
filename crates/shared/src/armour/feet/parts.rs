//! What a foot piece rolls about itself: which piece, and the shape of its toe.

use serde::{Deserialize, Serialize};

/// The three kinds of foot armour, all worn over a shoe or boot.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum FeetPiece {
    /// A toe cap, lames across the top of the foot and an ankle plate, laced to the
    /// shoe with a strap under it; sometimes a heel cap.
    Sabatons,
    /// Mail over the whole shoe, open under the sole with a strap, or soled.
    MailShoes,
    /// A plate, or plates on cloth or mail, over the instep and often the toes, tied
    /// on: kogake, and the Greek bronze feet.
    FootGuards,
}

impl FeetPiece {
    pub const ALL: [Self; 3] = [Self::Sabatons, Self::MailShoes, Self::FootGuards];
}

/// The shape of a sabaton's toe, which followed the shoes of its day (all but the claw
/// from the research: Sabaton, Poulaine and Duckbill shoe).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Toe {
    /// Following the foot: the one mail shoes and foot guards have.
    Round,
    /// Tapering to a point past the toes, 14th and 15th centuries.
    Pointed,
    /// Broad and rounded, wider than the foot.
    DuckBill,
    /// Broad, with lines running back from the toes.
    BearPaw,
    /// Widening abruptly at the toes, square across.
    CowMouth,
    /// Three curved claws (fantasy).
    Claw,
}

impl Toe {
    pub const ALL: [Self; 6] = [
        Self::Round,
        Self::Pointed,
        Self::DuckBill,
        Self::BearPaw,
        Self::CowMouth,
        Self::Claw,
    ];

    /// Wider than the foot.
    pub fn broad(self) -> bool {
        matches!(self, Self::DuckBill | Self::BearPaw | Self::CowMouth)
    }

    /// How wide it is at its widest, in hundredths of the foot's width: (least, most).
    pub fn widths(self) -> (u8, u8) {
        match self {
            Self::DuckBill => (150, 220),
            Self::BearPaw => (120, 150),
            Self::CowMouth => (120, 160),
            _ => (100, 100),
        }
    }
}

/// How far a pointed toe may reach past the toes, in millimetres: up to 130 fixed, up
/// to 250 when it comes off for fighting on foot (the owner's decision).
pub const POINT_MM: (u16, u16) = (20, 130);
pub const DETACHABLE_POINT_MM: (u16, u16) = (80, 250);
