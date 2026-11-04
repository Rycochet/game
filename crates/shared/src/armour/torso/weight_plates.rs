//! What armour of many small plates weighs, as a main weight at a reference density
//! and the rest. Small plates overlap more, so cover the 0.31 m² shell more thickly
//! (see the research for each figure).

use super::weight::{MAIL_UNDER, STEEL_DENSITY};
use super::{Backing, Lacing, LamellarCut, Mirrors, PlateRows, PlateSize, TorsoBody, TorsoForm};
use crate::material::Category;

/// Wood and rattan, the light references.
const WOOD_DENSITY: u32 = 7;
const RATTAN_DENSITY: u32 = 5;

impl TorsoForm {
    /// The main weight, its reference density and everything else, in grams.
    pub(super) fn plates_grams(&self) -> (u32, u32, u32) {
        match self.body {
            // About 1.4 times mail per m², more for small scales overlapping more.
            TorsoBody::Scale {
                size,
                backing,
                sleeves,
                ..
            } => {
                let scales = match size {
                    PlateSize::Small => 4200,
                    PlateSize::Medium => 3800,
                    PlateSize::Large => 3500,
                };
                let sleeves = if sleeves { scales / 7 } else { 0 };
                let backing = match backing {
                    Backing::Cloth => 300,
                    Backing::Leather => 500,
                    Backing::Mail => 2000,
                };
                (scales + sleeves, STEEL_DENSITY, backing)
            }
            // 13 kg/m² for iron, from a Ming set of about 20 kg.
            TorsoBody::Lamellar { cut, size, lacing } => {
                let plates = match size {
                    PlateSize::Small => 4600,
                    PlateSize::Medium => 4200,
                    PlateSize::Large => 3900,
                };
                let side = if cut == LamellarCut::Boxed { 400 } else { 0 };
                let laces = if lacing == Lacing::Full { 450 } else { 300 };
                (plates + side, STEEL_DENSITY, laces)
            }
            // Lames 1.2 mm, overlapping a fifth.
            TorsoBody::Laminar { lames, .. } => (lames as u32 * 600, STEEL_DENSITY, 300),
            TorsoBody::PlatesInCloth { size, .. } => {
                let plates = match size {
                    PlateSize::Small => 3600,
                    PlateSize::Medium => 3800,
                    PlateSize::Large => 3500,
                };
                (plates, STEEL_DENSITY, 500)
            }
            TorsoBody::MailAndPlate { rows, sleeves } => {
                let plates = match rows {
                    PlateRows::Wide => 2200,
                    PlateRows::Narrow => 2800,
                    PlateRows::Grid => 2000,
                };
                let sleeves = if sleeves { 500 } else { 0 };
                (MAIL_UNDER + plates + sleeves, STEEL_DENSITY, 0)
            }
            // The mail beneath is part of it.
            TorsoBody::Mirror { mirrors } => {
                let plates = match mirrors {
                    Mirrors::Discs => 900,
                    Mirrors::Four => 1400,
                };
                (MAIL_UNDER + plates, STEEL_DENSITY, 200)
            }
            // Slats 6 mm thick, or metal strips 1.2 mm with gaps.
            TorsoBody::Splint { .. } if self.material.category() == Category::Metal => {
                (2000, STEEL_DENSITY, 200)
            }
            TorsoBody::Splint { .. } => (1100, WOOD_DENSITY, 200),
            TorsoBody::Woven => (1500, RATTAN_DENSITY, 100),
            _ => (0, STEEL_DENSITY, 0),
        }
    }
}
