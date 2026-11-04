//! What a neck piece rolls about itself: which kind, how much a gorget covers, an
//! aventail's front, and the edges of mail and scales.

use serde::{Deserialize, Serialize};

/// The five kinds of neck piece, each its own item kind.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum NeckKind {
    /// Worn under the others, or alone, on the Middle layer: a padded collar, a mail
    /// standard or a Japanese collar lined with small plates.
    Collar,
    /// A rigid collar, alone or with plates over the chest, back and shoulders.
    Gorget,
    /// A cup over the chin and throat, in front, down to the top of the breast.
    Bevor,
    /// A curtain hung from a helmet's lower edge.
    Aventail,
    /// A short cape over the neck and shoulders: the bishop's mantle.
    Mantle,
}

impl NeckKind {
    pub const ALL: [Self; 5] = [
        Self::Collar,
        Self::Gorget,
        Self::Bevor,
        Self::Aventail,
        Self::Mantle,
    ];
}

/// How much of the chest, back and shoulders a gorget covers besides the neck.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Plates {
    /// The collar alone, a band round the neck. What every other kind has.
    Collar,
    /// The collar, with lames hanging over the top of the breast in front: the nodowa.
    Bib,
    /// The collar with plates over the top of the chest and back.
    Plates,
    /// The same with lames out over the tops of the shoulders, under the pauldrons.
    Shoulders,
}

impl Plates {
    pub const ALL: [Self; 4] = [Self::Collar, Self::Bib, Self::Plates, Self::Shoulders];
}

/// What an aventail covers at the front.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Front {
    /// The back and sides only; the face and throat show. What every other kind has.
    Open,
    /// Round the chin and throat, the face open: a bascinet's.
    Chin,
    /// Up over the lower face to the nose.
    Closed,
    /// Round the chin, with a flap that laces up over the mouth.
    Ventail,
}

impl Front {
    pub const ALL: [Self; 4] = [Self::Open, Self::Chin, Self::Closed, Self::Ventail];
}

/// The lower edge of mail, scales and lamellae.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Hem {
    Straight,
    /// Cut into points, zig-zag (mail only).
    Vandyked,
    /// The rounded ends of the last row (scales and lamellae only).
    Scalloped,
}

/// Rings of another metal in mail.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Edging {
    None,
    /// A band of brass rings round the edges.
    Brass,
    /// Bands of brass and copper rings through the iron, as on a kulah khud.
    Patterned,
}
