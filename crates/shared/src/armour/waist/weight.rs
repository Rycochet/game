//! What waist armour weighs and takes up. Weights are worked out from the area each
//! shape of piece covers on an ordinary person (at its distance from the skin) times how
//! heavy each construction is per area, scaled by the material's density, then checked
//! against a mail skirt of about 3 kg and a tonlet of about 5 to 6 kg (see the
//! research).

use super::{Length, WaistForm, WaistKind, WaistPiece};
use crate::armour::Construction;
use crate::creature::Size;
use crate::item::Shape;
use crate::material::Material;

/// Square centimetres of one fauld hoop, one culet hoop, and a mitra's plate.
const HOOP: u32 = 320;
const CULET_HOOP: u32 = 220;
const MITRA: u32 = 450;
/// A mail skirt's own area under plate, and a groin plate's.
const MAIL_BENEATH: u32 = 1200;
const GROIN: u32 = 150;

impl WaistForm {
    /// Square centimetres it covers, with overlaps.
    fn area(&self) -> u32 {
        let knee = self.length == Length::Knee;
        let mut area = match self.piece {
            WaistPiece::Girdle => 1100 + if self.front { MITRA } else { 0 },
            WaistPiece::Fauld => HOOP * self.lames.max(3) as u32,
            WaistPiece::Tassets => {
                let each = match self.kind {
                    // Wrapping the thigh, down over the knee.
                    WaistKind::Cuirassier => 1500,
                    _ if knee => 760,
                    _ => 440,
                };
                // The extra hangs are small: at the sides, or the Dendra's lower pairs.
                let extra = self.hangs.saturating_sub(2) as u32 * 250;
                HOOP * 3 + each * 2 + extra
            }
            WaistPiece::Panels => {
                let all = if knee { 6000 } else { 4500 };
                if self.hangs == 2 { all * 9 / 10 } else { all }
            }
            WaistPiece::Pteruges => {
                let long = if self.length == Length::Thigh {
                    1500
                } else {
                    0
                };
                let rows = self.lames.max(1) as u32 * 2000 + long;
                if self.front { rows * 4 / 10 } else { rows }
            }
            WaistPiece::Skirt => match (self.construction, knee) {
                // A tonlet belling out.
                (Construction::Plate, _) => 7300,
                (_, true) => 6000,
                (_, false) => 4000,
            },
            WaistPiece::Breeches => 3000,
        };
        area += CULET_HOOP * self.culet as u32;
        if self.groin {
            area += GROIN;
        }
        area
    }

    /// Milligrams per square centimetre of its main part, and the density in tenths of
    /// a gram per cm³ that was worked out for: plates about 1 mm, leather 4 mm.
    fn per_area(&self) -> (u32, u32) {
        const STEEL: u32 = 78;
        match self.construction {
            Construction::Plate => (750, STEEL),
            Construction::Leather => (400, 11),
            Construction::Mail => (750, STEEL),
            Construction::RingMail => (400, STEEL),
            Construction::MailAndPlate => (800, STEEL),
            Construction::Splint => (600, STEEL),
            Construction::Padded => (220, 15),
            // Scale, lamellar, laminar, plates in cloth: overlapping plates.
            _ => (950, STEEL),
        }
    }

    /// Milligrams per square centimetre of what it is built on: leather under splints,
    /// scales and rings, cloth over brigandine plates, and lacing.
    fn backing(&self) -> u32 {
        match self.construction {
            Construction::Splint | Construction::Scale | Construction::RingMail => 300,
            Construction::PlatesInCloth | Construction::MailAndPlate => 150,
            Construction::Lamellar => 60,
            _ => 0,
        }
    }

    /// The cells it takes up at an ordinary size, wide and long.
    pub fn shape(&self) -> Shape {
        let knee = self.length == Length::Knee;
        let (wide, long) = match self.piece {
            WaistPiece::Girdle => (3, 1),
            _ if self.kind == WaistKind::Cuirassier => (2, 4),
            WaistPiece::Tassets if knee => (2, 3),
            WaistPiece::Panels | WaistPiece::Skirt if knee && self.rigid() => (3, 3),
            WaistPiece::Panels | WaistPiece::Skirt if knee || self.rows() => (2, 3),
            _ => (2, 2),
        };
        Shape::rectangle(wide, long)
    }

    /// What it weighs at an ordinary size, in grams.
    pub fn grams(&self) -> u32 {
        let area = self.area();
        let (per_area, reference) = self.per_area();
        let density = match self.material {
            // Soft leather is worked thinner than hardened.
            Material::Leather => 8,
            material => material.density(),
        };
        let main = area * per_area / 1000 * density / reference;
        let backing = area * self.backing() / 1000;
        // Under plate, in the plate's metal.
        let mail = if self.mail_beneath {
            MAIL_BENEATH * 750 / 1000
        } else {
            0
        };
        let extras = if self.studded { 60 } else { 0 }
            + if self.trim.is_some() { 60 } else { 0 }
            // Straps, buckles, cord and points.
            + 60;
        main + backing + mail + extras
    }

    /// What it weighs made at this size: bigger across, as thick (the owner's
    /// decision), so by the square of its size.
    pub fn grams_at(&self, size: Size) -> u32 {
        let quarters = size.quarters() as u64;
        (self.grams() as u64 * quarters * quarters / 16) as u32
    }
}
