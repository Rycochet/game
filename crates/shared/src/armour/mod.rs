//! Armour: what every piece and set shares. Pieces for each body location are added by
//! their own work, built on `body::Frame`. See `docs/context/shared/armour.md`.

mod finish;
pub mod hands;
pub mod shoulders;
pub mod torso;

use serde::{Deserialize, Serialize};

use crate::rng::Rng;

pub use finish::{Dye, Finish};

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
    /// Rings sewn flat on leather (the owner kept it, though doubted historically).
    /// Saved by name, so it goes at the end.
    RingMail,
}

impl Construction {
    pub const ALL: [Self; 12] = [
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
        Self::RingMail,
    ];

    /// Whether it bends with the body rather than holding its shape.
    pub fn flexible(self) -> bool {
        matches!(
            self,
            Self::Padded | Self::Mail | Self::Scale | Self::Lamellar | Self::RingMail
        )
    }
}

/// How a piece looks, shared by every piece of a set so they match: the owner's list of
/// harness styles, historical and fantasy, and regional looks. Each construction is made
/// only in some styles (`allows`). European styles mostly shape plate; on other
/// constructions they change only trim and edges. Saved by name, so new ones go at the
/// end.
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
    /// Lacquered lamellae and lames laced in silk: sode, dō, kote.
    Japanese,
    /// Lamellae and scales, beast heads on the shoulders, mirrors on the breast.
    Chinese,
    /// Mongol and Turkic: lamellar and laminar of iron and rawhide.
    Steppe,
    /// Greek and Roman: bronze, linen, segmented lames, mail doublings.
    Classical,
    /// Lamellar and splinted, over mail.
    Byzantine,
    /// Mail and plate, and mirrors.
    IndoPersian,
}

impl Style {
    pub const ALL: [Self; 14] = [
        Self::White,
        Self::Gothic,
        Self::Milanese,
        Self::Maximilian,
        Self::Greenwich,
        Self::Kastenbrust,
        Self::Elven,
        Self::Dwarven,
        Self::Japanese,
        Self::Chinese,
        Self::Steppe,
        Self::Classical,
        Self::Byzantine,
        Self::IndoPersian,
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
            Self::Japanese => "Japanese",
            Self::Chinese => "Chinese",
            Self::Steppe => "Steppe",
            Self::Classical => "Classical",
            Self::Byzantine => "Byzantine",
            Self::IndoPersian => "Indo-Persian",
        }
    }

    /// One of the European harnesses, which mostly shape plate.
    pub fn european(self) -> bool {
        matches!(
            self,
            Self::White
                | Self::Gothic
                | Self::Milanese
                | Self::Maximilian
                | Self::Greenwich
                | Self::Kastenbrust
        )
    }

    /// Whether this construction is made in this style (the owner's decision: each
    /// construction allows only some styles). Padding goes with anything; the fantasy
    /// styles with all but mirrors.
    pub fn allows(self, construction: Construction) -> bool {
        use Construction as C;
        match construction {
            C::Padded => true,
            _ if self.european() => matches!(
                construction,
                C::Leather
                    | C::Mail
                    | C::RingMail
                    | C::Scale
                    | C::PlatesInCloth
                    | C::Splint
                    | C::Plate
            ),
            _ => match self {
                Self::Elven | Self::Dwarven => construction != C::Mirror,
                Self::Japanese => !matches!(construction, C::Scale | C::Mirror),
                Self::Chinese => matches!(
                    construction,
                    C::Leather | C::Mail | C::Scale | C::Lamellar | C::PlatesInCloth | C::Mirror
                ),
                Self::Steppe => matches!(
                    construction,
                    C::Leather
                        | C::Mail
                        | C::Scale
                        | C::Lamellar
                        | C::Laminar
                        | C::MailAndPlate
                        | C::Mirror
                ),
                Self::Classical => matches!(
                    construction,
                    C::Leather | C::Mail | C::Scale | C::Laminar | C::Plate
                ),
                Self::Byzantine => matches!(
                    construction,
                    C::Leather | C::Mail | C::Scale | C::Lamellar | C::Laminar | C::Splint
                ),
                _ => matches!(
                    construction,
                    C::Mail | C::Scale | C::Lamellar | C::MailAndPlate | C::Mirror | C::Plate
                ),
            },
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
            (Self::Japanese, 8),
            (Self::Chinese, 6),
            (Self::Steppe, 6),
            (Self::Classical, 6),
            (Self::Byzantine, 4),
            (Self::IndoPersian, 4),
        ])
    }
}

/// The constructions in a list that a style allows, with their weights; the whole list
/// if it allows none, so every kind of piece can be made in every style.
pub fn in_style(list: &[(Construction, u64)], style: Style) -> Vec<(Construction, u64)> {
    let allowed: Vec<_> = list
        .iter()
        .copied()
        .filter(|&(construction, _)| style.allows(construction))
        .collect();
    if allowed.is_empty() {
        list.to_vec()
    } else {
        allowed
    }
}
