//! What arm armour weighs and takes up. Weights are worked out from the area of each
//! part of one arm (an ordinary person's, at the piece's distance from the skin) times
//! how heavy each construction is per area, scaled by the material's density (see the
//! research): a steel arm harness for both arms comes to about 3.3 kg.

use super::{ArmsForm, Couter, Reach, Sides, Spikes};
use crate::armour::Construction;
use crate::creature::Size;
use crate::item::Shape;
use crate::material::Material;

/// Square centimetres of one arm each part covers.
const WRIST: u32 = 260;
const FOREARM: u32 = 720;
const ELBOW: u32 = 400;
const UPPER: u32 = 610;

impl ArmsForm {
    /// Square centimetres of one arm it covers, with overlaps and a couter's fan.
    fn area(&self) -> u32 {
        let elbow = if self.reach.elbow() { ELBOW } else { 0 };
        let area = match self.reach {
            Reach::Wrist => WRIST,
            _ => {
                let forearm = if self.reach.forearm() { FOREARM } else { 0 };
                let upper = if self.reach.upper() { UPPER } else { 0 };
                forearm + elbow + upper
            }
        };
        let fan = if self.couter == Couter::Winged {
            elbow * 15 / 100
        } else {
            0
        };
        // Lames and edges overlap by about a tenth on longer pieces.
        let overlap = if self.reach == Reach::Whole {
            area / 10
        } else {
            0
        };
        area + fan + overlap
    }

    /// Milligrams per square centimetre of its main part, and the density in tenths of
    /// a gram per cm³ that was worked out for: plates about 1 mm, leather 4 mm.
    fn per_area(&self) -> (u32, u32) {
        const STEEL: u32 = 78;
        match self.construction {
            Construction::Plate => (870, STEEL),
            Construction::Leather => (400, 11),
            Construction::Mail => (750, STEEL),
            Construction::RingMail => (400, STEEL),
            Construction::MailAndPlate => (800, STEEL),
            Construction::Splint => (600, STEEL),
            Construction::Padded => (220, 15),
            // Scale, lamellar, laminar, plates in cloth: overlapping plates.
            _ => (1000, STEEL),
        }
    }

    /// Milligrams per square centimetre of what it is built on: leather under splints,
    /// scales and rings, cloth under kote and over brigandine plates.
    fn backing(&self) -> u32 {
        match self.construction {
            Construction::Splint | Construction::Scale | Construction::RingMail => 300,
            Construction::PlatesInCloth | Construction::MailAndPlate => 150,
            _ => 0,
        }
    }

    /// The cells it takes up at an ordinary size, wide and long.
    pub fn shape(&self) -> Shape {
        let (wide, long) = match self.reach {
            Reach::Wrist if self.sides != Sides::Both => (1, 1),
            Reach::Wrist => (1, 2),
            Reach::Elbow => (2, 1),
            Reach::Whole if self.sides != Sides::Both => (1, 3),
            Reach::Whole if self.construction == Construction::Mail => (2, 2),
            Reach::Whole => (2, 3),
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
        let extras = match self.spikes {
            Spikes::None => 0,
            _ => 60,
        } + if self.studded { 40 } else { 0 }
            + if self.trim.is_some() { 50 } else { 0 }
            // Straps and buckles.
            + 40;
        (main + backing + extras) * self.sides.count()
    }

    /// What it weighs made at this size: bigger across, as thick (the owner's
    /// decision), so by the square of its size.
    pub fn grams_at(&self, size: Size) -> u32 {
        let quarters = size.quarters() as u64;
        (self.grams() as u64 * quarters * quarters / 16) as u32
    }
}
