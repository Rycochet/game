//! What a head piece weighs and takes up. A helmet's weight is worked out from the area
//! of its parts in cm² (a bowl to the ears about 900 on a human head), times thickness
//! and density, plus its lining. Checked against a great helm of 1.5 to 3 kg and the
//! Sutton Hoo helmet at about 2.5 kg (see the research).

use super::{Back, Brim, Face, HeadForm, HeadKind, Ornament, Skull, Top, Visor};
use crate::armour::Construction;
use crate::creature::Size;
use crate::item::Shape;
use crate::material::Material;

/// Steel's and linen's density, in tenths of a gram per cm³, that the soft pieces'
/// weights are given in.
const STEEL_DENSITY: u32 = 78;
const LINEN_DENSITY: u32 = 15;
/// Every helmet's lining and straps.
const LINING: u32 = 200;

impl HeadForm {
    /// The cells it takes up at an ordinary size, wide and long.
    pub fn shape(&self) -> Shape {
        let (wide, long) = match self.kind {
            HeadKind::Cap => (2, 1),
            HeadKind::Coif => (2, 2),
            HeadKind::BevoredHelm => (2, 3),
            HeadKind::Helmet => match (self.brim, self.ornament, self.top) {
                (Brim::Wide, _, _) => (3, 2),
                (_, Ornament::Horns | Ornament::Wings | Ornament::Antlers, _) => (3, 2),
                (_, _, Top::Plume) => (2, 3),
                _ if self.skull_cap() && self.top == Top::None => (2, 1),
                _ => (2, 2),
            },
        };
        Shape::rectangle(wide, long)
    }

    /// Its parts' area in cm².
    fn area(&self) -> u32 {
        let bowl = match self.skull {
            Skull::Round => 900,
            Skull::Low => 700,
            Skull::Conical | Skull::Forward => 950,
            Skull::Ogival => 1100,
            Skull::Flat => 1300,
            Skull::Onion => 1000,
        };
        let face = match self.face {
            Face::Open => 0,
            Face::Nasal => 40,
            Face::Bars => 60,
            Face::Spectacles => 120,
            Face::Mask => 400,
            Face::Opening => 450,
            Face::Faceplate | Face::Slit => 500,
            Face::Visor(Visor::Klapp) => 300,
            Face::Visor(Visor::Rounded) => 600,
            Face::Visor(Visor::Pointed | Visor::Grotesque) => 650,
            Face::Visor(Visor::Bellows) => 700,
        };
        let cheeks = if self.cheeks { 240 } else { 0 };
        let back = match self.back {
            Back::None => 0,
            Back::Flare => 80,
            Back::Guard => 200,
            Back::Tail => 350,
            Back::Lames => 400,
            Back::Flaps => 500,
        };
        let brim = match self.brim {
            Brim::None => 0,
            Brim::Peak => 100,
            Brim::Narrow => 250,
            Brim::Boat => 600,
            Brim::Wide => 700,
        };
        let top = match self.top {
            Top::Knob | Top::Stalk => 20,
            Top::Spike => 40,
            Top::Comb => 60,
            Top::HighComb => 200,
            Top::None | Top::Crest | Top::Plume => 0,
        };
        let bevor = if self.kind == HeadKind::BevoredHelm {
            600
        } else {
            0
        };
        bowl + face + cheeks + back + brim + top + bevor
    }

    /// Thickness in hundredths of a millimetre, with lamellar's overlap counted in.
    fn thickness(&self) -> u32 {
        let jousting = if self.face == Face::Slit && self.kind == HeadKind::BevoredHelm {
            2
        } else {
            1
        };
        jousting
            * match (self.construction, self.material) {
                (_, Material::Bronze | Material::Copper) => 120,
                (Construction::Lamellar, _) => 240,
                (Construction::Leather, _) => 400,
                (Construction::Woven, _) => 800,
                _ => 200,
            }
    }

    /// What it weighs at an ordinary size, in grams.
    pub fn grams(&self) -> u32 {
        let density = self.material.density();
        match self.kind {
            HeadKind::Cap => 250 * density / LINEN_DENSITY,
            HeadKind::Coif => match self.construction {
                // About 6 kg a square metre over 0.35 m².
                Construction::Mail => 2100 * density / STEEL_DENSITY,
                Construction::RingMail => 1200 * density / STEEL_DENSITY + 600,
                _ => 500 * density / LINEN_DENSITY,
            },
            HeadKind::Helmet | HeadKind::BevoredHelm => {
                let shell = self.area() * self.thickness() * density / 10_000;
                let extras = match self.top {
                    Top::Crest => 150,
                    Top::Plume => 60,
                    _ => 0,
                } + match self.ornament {
                    Ornament::None => 0,
                    Ornament::FrontCrest | Ornament::Crown => 150,
                    Ornament::Wings => 300,
                    Ornament::Horns | Ornament::Antlers => 400,
                    Ornament::Dragon => 500,
                } + if self.studded { 100 } else { 0 }
                    + if self.tusks() { 300 } else { 0 }
                    + if self.lacing.is_some() { 100 } else { 0 };
                shell + LINING + extras
            }
        }
    }

    /// What it weighs made at this size: bigger across, as thick (the owner's
    /// decision), so by the square of its size.
    pub fn grams_at(&self, size: Size) -> u32 {
        let quarters = size.quarters() as u64;
        (self.grams() as u64 * quarters * quarters / 16) as u32
    }
}
