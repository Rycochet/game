//! What leg armour weighs and takes up. Weights are worked out from the area of each
//! part of one leg (an ordinary person's, at the piece's distance from the skin) times
//! how heavy each construction is per area, scaled by the material's density, then
//! checked against a surviving cuisse with poleyn of about 1.2 kg a leg (Fitzwilliam
//! Museum) and steel plate legs of about 4 kg a pair (see the research).

use super::{Cover, Knee, LegsForm, Ornament, Reinforce};
use crate::armour::Construction;
use crate::creature::Size;
use crate::item::Shape;
use crate::material::Material;

/// Square centimetres of one leg each part covers, all round.
const THIGH: u32 = 900;
const KNEE: u32 = 380;
const SHIN: u32 = 1100;
const ANKLE: u32 = 250;
const FOOT: u32 = 350;

impl LegsForm {
    /// Square centimetres of one leg it covers, with overlaps and a knee's fan.
    fn area(&self) -> u32 {
        let cover = self.cover;
        let mut area = match cover {
            Cover::Ankle => ANKLE,
            _ => {
                let thigh = if cover.thigh() { THIGH } else { 0 };
                let knee = if cover.knee() { KNEE } else { 0 };
                let shin = if cover.shin() { SHIN } else { 0 };
                thigh + knee + shin
            }
        };
        if !self.closed {
            // Over the front and outside only.
            area = area * 6 / 10;
        }
        if self.knee == Knee::Winged {
            area += KNEE * 15 / 100;
        }
        if cover == Cover::Whole {
            // Lames and edges overlap by about a tenth.
            area += area / 10;
        }
        if self.footed() {
            area += FOOT;
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
    /// scales and rings, cloth under Japanese pieces and over brigandine plates.
    fn backing(&self) -> u32 {
        match self.construction {
            Construction::Splint | Construction::Scale | Construction::RingMail => 300,
            Construction::PlatesInCloth | Construction::MailAndPlate => 150,
            _ if self.japanese() => 150,
            _ => 0,
        }
    }

    /// Grams on one leg of plates strapped over mail.
    fn reinforcements(&self) -> u32 {
        match self.reinforce {
            Reinforce::None => 0,
            Reinforce::Poleyns => 300,
            Reinforce::PoleynsAndSchynbalds => 750,
            Reinforce::QuiltedThighs => 550,
        }
    }

    /// The cells it takes up at an ordinary size, wide and long: the two legs side by
    /// side, mail and padding folded.
    pub fn shape(&self) -> Shape {
        let one = self.sides.count() == 1;
        let (wide, long) = match self.cover {
            Cover::Knee | Cover::Ankle => (2, 1),
            Cover::Shin | Cover::Thigh if one => (1, 2),
            Cover::Shin | Cover::Thigh => (2, 2),
            Cover::KneeAndShin if one => (1, 3),
            Cover::ThighAndKnee | Cover::KneeAndShin => (2, 3),
            Cover::Whole if self.construction.flexible() => (2, 3),
            Cover::Whole => (2, 4),
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
        let extras = match self.ornament {
            Ornament::None => 0,
            _ => 70,
        } + if self.studded { 50 } else { 0 }
            + if self.trim.is_some() { 60 } else { 0 }
            // Straps, buckles and points.
            + 40;
        (main + backing + self.reinforcements() + extras) * self.sides.count()
    }

    /// What it weighs made at this size: bigger across, as thick (the owner's
    /// decision), so by the square of its size.
    pub fn grams_at(&self, size: Size) -> u32 {
        let quarters = size.quarters() as u64;
        (self.grams() as u64 * quarters * quarters / 16) as u32
    }
}
