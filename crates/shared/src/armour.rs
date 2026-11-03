//! Armour: what every piece and set shares. Pieces for each body location are added by
//! their own work, built on `body::Frame`. See `docs/context/shared/armour.md`.

use serde::{Deserialize, Serialize};

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
