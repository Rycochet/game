//! What foot armour weighs and takes up. Weights are worked out from the area each
//! piece covers on one foot times how heavy its construction is per area (the legs'
//! figures), scaled by the material's density, then checked against a German sabaton
//! of about 1490 that weighs 7 oz, about 200 g, with no heel cap (Higgins Armory
//! Museum's label; see the research).

use super::{FeetForm, FeetPiece, Toe};
use crate::armour::Construction;
use crate::creature::Size;
use crate::item::Shape;
use crate::material::Material;

/// Square centimetres of one foot each part covers.
const TOP: u32 = 260;
const HEEL: u32 = 200;
const CUFF: u32 = 200;
const SHOE: u32 = 450;
const INSTEP: u32 = 130;
const TOES: u32 = 60;

impl FeetForm {
    /// Square centimetres of one foot it covers.
    fn area(&self) -> u32 {
        match self.piece {
            FeetPiece::Sabatons => {
                let mut area = TOP * self.width as u32 / 100;
                if self.toe == Toe::Pointed {
                    // A cone over the top only, about 8 mm² for each millimetre out.
                    area += self.point as u32 * 8 / 10;
                }
                if self.heel {
                    area += HEEL;
                }
                if self.high {
                    area += CUFF;
                }
                area
            }
            FeetPiece::MailShoes => SHOE,
            FeetPiece::FootGuards => INSTEP + if self.toes { TOES } else { 0 },
        }
    }

    /// Milligrams per square centimetre of its main part, and the density in tenths of
    /// a gram per cm³ that was worked out for: plates about 1 mm, leather 4 mm (as the
    /// legs' pieces).
    fn per_area(&self) -> (u32, u32) {
        const STEEL: u32 = 78;
        match self.construction {
            Construction::Plate => (750, STEEL),
            Construction::Leather => (400, 11),
            Construction::Mail => (750, STEEL),
            Construction::RingMail => (400, STEEL),
            Construction::MailAndPlate => (800, STEEL),
            Construction::Splint => (600, STEEL),
            // Scales and plates in cloth: overlapping plates.
            _ => (950, STEEL),
        }
    }

    /// Milligrams per square centimetre of what it is built on: leather under splints,
    /// scales and rings, cloth under Japanese pieces, and a soled mail shoe's sole.
    fn backing(&self) -> u32 {
        match self.construction {
            Construction::Splint | Construction::Scale | Construction::RingMail => 300,
            Construction::PlatesInCloth | Construction::MailAndPlate => 150,
            _ if self.japanese() => 150,
            _ => 0,
        }
    }

    /// The cells it takes up at an ordinary size, wide and long: rigid sabatons as
    /// boots, the rest folded flat (the owner's decision).
    pub fn shape(&self) -> Shape {
        match self.piece {
            FeetPiece::Sabatons => Shape::rectangle(2, 2),
            _ => Shape::rectangle(2, 1),
        }
    }

    /// What the pair weighs at an ordinary size, in grams.
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
        let extras = if self.studded { 30 } else { 0 }
            + if self.trim.is_some() { 30 } else { 0 }
            + if self.soled { 80 } else { 0 }
            // Laces, straps and buckles.
            + 20;
        (main + backing + extras) * 2
    }

    /// What it weighs made at this size: bigger across, as thick (the owner's
    /// decision), so by the square of its size.
    pub fn grams_at(&self, size: Size) -> u32 {
        let quarters = size.quarters() as u64;
        (self.grams() as u64 * quarters * quarters / 16) as u32
    }
}
