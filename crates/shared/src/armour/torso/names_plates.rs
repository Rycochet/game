//! Specific names for armour of many small plates, when one fits.

use super::{
    Backing, LamellarCut, Mirrors, PlateRows, PlateSize, Rivets, ScaleShape, TorsoBody, TorsoForm,
};
use crate::armour::Style;
use crate::material::{Category, Material};

impl TorsoForm {
    /// A name from history or stories for scale, lamellar, laminar, plates in cloth,
    /// plated mail, mirrors and slats.
    pub(super) fn plates_name(&self) -> Option<&'static str> {
        let style = self.style;
        match self.body {
            TorsoBody::Scale { shape, backing, .. } => match (shape, self.material) {
                (ScaleShape::Coin, _) => Some("Coin Armour"),
                (_, Material::DragonScale) => Some("Dragonscale Shirt"),
                (_, Material::Horn) => Some("Horn Scale Shirt"),
                _ if style == Style::Classical => Some(if backing == Backing::Mail {
                    "Lorica Plumata"
                } else {
                    "Lorica Squamata"
                }),
                _ => None,
            },
            TorsoBody::Lamellar { cut, .. } => match (cut, style) {
                (LamellarCut::Wrap, Style::Japanese) => Some("Dō-maru"),
                (LamellarCut::OpenBack, _) => Some("Haramaki"),
                (LamellarCut::Boxed, _) => Some("Ō-yoroi Dō"),
                (LamellarCut::Mountain, _) => Some("Mountain Pattern Cuirass"),
                _ if self.material == Material::Bone => Some("Bone Lamellar"),
                (_, Style::Byzantine) => Some("Klivanion"),
                _ => None,
            },
            TorsoBody::Laminar { riveted, .. } => {
                if self.over_mail {
                    Some("Banded Mail")
                } else if style == Style::Classical {
                    Some("Lorica Segmentata")
                } else if style == Style::Japanese && riveted {
                    Some("Okegawa-dō")
                } else {
                    None
                }
            }
            TorsoBody::PlatesInCloth {
                size,
                rivets,
                leather,
            } => {
                if leather {
                    Some("Leather Brigandine")
                } else if rivets == Rivets::Hidden {
                    Some("Jack of Plate")
                } else if style == Style::Chinese {
                    Some("Dingjia")
                } else if style == Style::Japanese {
                    Some("Tatami Dō")
                } else if size == PlateSize::Large {
                    Some("Coat of Plates")
                } else {
                    None
                }
            }
            TorsoBody::MailAndPlate { rows, .. } => match (rows, style) {
                (PlateRows::Wide, Style::Steppe) => Some("Yushman"),
                (PlateRows::Narrow, Style::Steppe) => Some("Bakhterets"),
                (PlateRows::Grid, Style::IndoPersian) => Some("Zirah Bagtar"),
                _ => None,
            },
            TorsoBody::Mirror {
                mirrors: Mirrors::Four,
            } => Some("Char-aina"),
            TorsoBody::Splint { .. } if self.material.category() != Category::Metal => {
                Some("Slat Armour")
            }
            _ => None,
        }
    }
}
