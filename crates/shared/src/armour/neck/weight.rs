//! What a neck piece weighs and takes up. Weights are worked out from the area each
//! part covers on an ordinary body times how heavy its construction is per area (a
//! plate gorget's 1.5 mm, a bevor's 2 mm, mail at about 6.5 kg a square metre),
//! scaled by the material's density, then checked against museum figures: gorgets
//! about 0.8 to 1.5 kg, sallet bevors about 1 kg, aventails 1 to 2 kg (see the
//! research).

use super::{Front, NeckForm, NeckKind, Plates};
use crate::armour::Construction;
use crate::creature::Size;
use crate::item::Shape;
use crate::material::Material;

/// Square centimetres each part covers.
const COLLAR_HIGH: u32 = 350;
const COLLAR_LOW: u32 = 200;
const BIB: u32 = 250;
const PLATES: u32 = 550;
const SHOULDER_LAMES: u32 = 400;
const BEVOR_MOUTH: u32 = 650;
const BEVOR_CHIN: u32 = 500;
const BEVOR_LAME: u32 = 80;
const STANDARD: u32 = 1000;
const SOFT_COLLAR: u32 = 600;
const MANTLE: u32 = 3000;

impl NeckForm {
    /// Square centimetres it covers.
    fn area(&self) -> u32 {
        let collar = if self.high { COLLAR_HIGH } else { COLLAR_LOW };
        let cape = |area: u32| if self.cape { area * 3 / 2 } else { area };
        match self.kind {
            NeckKind::Collar => match self.construction {
                Construction::Mail | Construction::RingMail => cape(STANDARD),
                _ => SOFT_COLLAR,
            },
            NeckKind::Gorget => match self.plates {
                Plates::Collar => collar,
                Plates::Bib => collar + BIB,
                Plates::Plates => collar + PLATES,
                Plates::Shoulders => collar + PLATES + SHOULDER_LAMES,
            },
            NeckKind::Bevor => {
                let cup = if self.high { BEVOR_MOUTH } else { BEVOR_CHIN };
                cup + BEVOR_LAME * self.lames as u32
            }
            NeckKind::Aventail => cape(match self.front {
                Front::Open => 1300,
                Front::Chin => 1700,
                Front::Ventail => 1800,
                Front::Closed => 2000,
            }),
            NeckKind::Mantle => MANTLE,
        }
    }

    /// Milligrams per square centimetre of its main part, and the density in tenths of
    /// a gram per cm³ that was worked out for; `None` for padding, which is as heavy
    /// whatever cloth covers it.
    fn per_area(&self) -> Option<(u32, u32)> {
        const STEEL: u32 = 78;
        Some(match self.construction {
            Construction::Plate if self.kind == NeckKind::Bevor => (1570, STEEL),
            Construction::Plate => (1180, STEEL),
            Construction::Leather => (400, 11),
            Construction::Mail => (650, STEEL),
            Construction::RingMail => (400, STEEL),
            // Hanging curtains of smaller, thinner scales than a gorget's.
            Construction::Scale if self.kind != NeckKind::Gorget => (700, STEEL),
            Construction::Scale => (950, STEEL),
            Construction::Lamellar => (1100, STEEL),
            Construction::Laminar => (900, STEEL),
            Construction::MailAndPlate => (800, STEEL),
            // Kikko: small plates spaced out in the cloth.
            Construction::PlatesInCloth => (500, STEEL),
            _ => return None,
        })
    }

    /// Milligrams per square centimetre of what it is built on or held by: leather
    /// under scales and rings, cloth round kikko, laces through lamellae.
    fn backing(&self) -> u32 {
        match self.construction {
            Construction::Scale | Construction::RingMail => 300,
            Construction::PlatesInCloth => 150,
            Construction::Lamellar | Construction::Laminar => 50,
            Construction::Padded => 250,
            _ => 0,
        }
    }

    /// The cells it takes up at an ordinary size, wide and long: collars 2×1, the rest
    /// 2×2 (the owner's decision).
    pub fn shape(&self) -> Shape {
        match (self.kind, self.plates) {
            (NeckKind::Collar, _) | (NeckKind::Gorget, Plates::Collar) => Shape::rectangle(2, 1),
            _ => Shape::rectangle(2, 2),
        }
    }

    /// What it weighs at an ordinary size, in grams.
    pub fn grams(&self) -> u32 {
        let area = self.area();
        let main = self.per_area().map_or(0, |(per_area, reference)| {
            let density = match self.material {
                // Soft leather is worked thinner than hardened.
                Material::Leather => 8,
                material => material.density(),
            };
            area * per_area / 1000 * density / reference
        });
        let backing = area * self.backing() / 1000;
        let fittings = if self.rigid() { 100 } else { 50 };
        let extras = if self.studded { 30 } else { 0 }
            + if self.trim.is_some() { 30 } else { 0 }
            + if self.spikes { 80 } else { 0 };
        main + backing + fittings + extras
    }

    /// What it weighs made at this size: bigger across, as thick (the owner's
    /// decision), so by the square of its size.
    pub fn grams_at(&self, size: Size) -> u32 {
        let quarters = size.quarters() as u64;
        (self.grams() as u64 * quarters * quarters / 16) as u32
    }
}
