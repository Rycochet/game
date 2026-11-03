//! What torso armour weighs and takes up. Weights are worked out from the frame: a
//! human torso shell from waist to shoulders is about 0.31 m² (breast 0.17, back 0.14),
//! times thickness and density (see the research).

use super::{Breast, Plackart, Quilting, Rings, TorsoBody, TorsoForm, Weave};
use crate::creature::Size;
use crate::item::Shape;

/// What a main weight is worked out against, in tenths of a gram per cm³.
const STEEL_DENSITY: u32 = 78;
const HARD_LEATHER_DENSITY: u32 = 11;
const LINEN_DENSITY: u32 = 15;
/// A mail shirt with short sleeves, 0.36 m² at 8.5 kg/m², worn under a rigid piece.
const MAIL_UNDER: u32 = 3100;

impl TorsoForm {
    /// The cells it takes up at an ordinary size, wide and long.
    pub fn shape(&self) -> Shape {
        let (wide, long) =
            match self.body {
                _ if self.vest => (2, 2),
                TorsoBody::Plate {
                    breast: Breast::Discs,
                    ..
                } => (2, 2),
                TorsoBody::Plate { .. } | TorsoBody::Leather { .. } if self.rigid() => {
                    if self.back { (3, 3) } else { (2, 2) }
                }
                TorsoBody::Mail { .. } => (2, 2),
                _ => (2, 3),
            };
        Shape::rectangle(wide, long)
    }

    /// What it weighs at an ordinary size, in grams: its main material's share by that
    /// material's density, then everything else.
    pub fn grams(&self) -> u32 {
        let (main, reference, rest) = match self.body {
            TorsoBody::Plate {
                breast, plackart, ..
            } => {
                // Breast 2.5 mm, back 1.5 mm, by the shape's share of metal.
                let (breast_grams, back_grams) = match breast {
                    Breast::Discs => (900, 900),
                    Breast::Munition => (4300, 2000),
                    Breast::Panel => (4200, 2100),
                    Breast::Leaf => (2800, 1400),
                    Breast::Tanko => (2600, 2000),
                    _ => (3300, 1650),
                };
                let plackart = match plackart {
                    Plackart::None => 0,
                    Plackart::Half => 500,
                    Plackart::Pointed => 600,
                };
                let back = if self.back { back_grams } else { 0 };
                (breast_grams + back + plackart, STEEL_DENSITY, 300)
            }
            // 5 mm hardened leather, or 4 mm buff.
            TorsoBody::Leather { .. } if self.rigid() => {
                let back = if self.back { 800 } else { 0 };
                (1000 + back, HARD_LEATHER_DENSITY, 200)
            }
            TorsoBody::Leather { .. } => (1250, 9, 150),
            TorsoBody::Mail {
                weave,
                rings,
                sleeves,
                ..
            } => {
                let base = match rings {
                    Rings::Fine => 3400,
                    Rings::Medium => 3100,
                    Rings::Coarse => 2800,
                };
                let base = match weave {
                    Weave::FourInOne => base,
                    Weave::SixInOne => base * 9 / 5,
                    Weave::Japanese => base * 3 / 5,
                };
                let sleeves = if sleeves { base / 6 } else { 0 };
                let cloth = if weave == Weave::Japanese { 400 } else { 0 };
                (base + sleeves, STEEL_DENSITY, cloth)
            }
            TorsoBody::RingMail { sleeves } => {
                let sleeves = if sleeves { 400 } else { 0 };
                (1500 + sleeves, STEEL_DENSITY, 1500)
            }
            TorsoBody::Padded { quilting, sleeves } => {
                let base = match quilting {
                    _ if self.vest => 1200,
                    Quilting::Laminated => 3000,
                    _ => 2500,
                };
                let sleeves = if sleeves { 400 } else { 0 };
                (base + sleeves, LINEN_DENSITY, 0)
            }
        };
        let studded = matches!(self.body, TorsoBody::Leather { studded: true, .. });
        let extras = if self.over_mail { MAIL_UNDER } else { 0 }
            + if self.lined { 900 } else { 0 }
            + if studded { 300 } else { 0 }
            + if self.trim.is_some() { 100 } else { 0 }
            + match self.body {
                TorsoBody::Plate {
                    lance_rest: true, ..
                } => 250,
                _ => 0,
            };
        main * self.material.density() / reference + rest + extras
    }

    /// What it weighs made at this size: bigger across, as thick (the owner's
    /// decision), so by the square of its size.
    pub fn grams_at(&self, size: Size) -> u32 {
        let quarters = size.quarters() as u64;
        (self.grams() as u64 * quarters * quarters / 16) as u32
    }
}
