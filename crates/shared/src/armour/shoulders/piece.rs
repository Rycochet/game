//! `ShouldersPiece` and `ShouldersKind`: what is worn on the shoulders.

use serde::{Deserialize, Serialize};

use super::{
    AiletteShape, CapeCut, Cut, GuardShape, HautePiece, Heraldry, Lacing, Length, Span, Wings,
};

/// Which kind of shoulder piece, with what only that kind has.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ShouldersPiece {
    /// A dome over the joint (the cop), wings over the breast and back, and lames down
    /// the arm: covers the armpit.
    Pauldrons {
        cut: Cut,
        wings: Wings,
        /// Lames below the cop, and up towards the neck.
        lames: u8,
        upper_lames: u8,
        haute_piece: HautePiece,
        /// A second plate over the front of the cop.
        gardbrace: bool,
        /// Discs hung in front of the armpits.
        besagews: bool,
    },
    /// A smaller cop and lames over the top and outside of the shoulder; the armpit is
    /// open. Laminar ones are Roman, bronze plate ones Mycenaean (Dendra).
    Spaulders {
        lames: u8,
        besagews: bool,
        /// Rows of hanging strips below them.
        pteruges: u8,
    },
    /// A panel hanging from the shoulder over the upper arm, in rows.
    Guards {
        size: Span,
        shape: GuardShape,
        rows: u8,
        lacing: Lacing,
    },
    /// One piece over both shoulders round the neck.
    Cape { cut: CapeCut, length: Length },
    /// Round plates on the outside of the shoulders.
    Discs {
        /// Across, in millimetres.
        diameter: u16,
        boss: bool,
    },
    /// Flat boards standing on the shoulders, painted.
    Ailettes {
        shape: AiletteShape,
        size: Span,
        heraldry: Heraldry,
    },
    /// One huge board over the left shoulder down to the waist, used as a shield.
    Wing,
}

/// The kinds of shoulder piece, without what each has.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum ShouldersKind {
    Pauldrons,
    Spaulders,
    Guards,
    Cape,
    Discs,
    Ailettes,
    Wing,
}

impl ShouldersKind {
    pub const ALL: [Self; 7] = [
        Self::Pauldrons,
        Self::Spaulders,
        Self::Guards,
        Self::Cape,
        Self::Discs,
        Self::Ailettes,
        Self::Wing,
    ];
}
