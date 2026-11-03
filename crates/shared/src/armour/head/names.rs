//! What a head piece is called: the first named row it fits, or a generic name from
//! its shape, with its material first when that is unusual and an ornament first of all.

use super::table::ROWS;
use super::{Face, HeadForm, HeadKind, Visor};
use crate::armour::Construction;
use crate::material::Material;

impl HeadForm {
    /// A name from history or stories, when it fits one.
    pub fn specific_name(&self) -> Option<&'static str> {
        if !self.helmet() {
            return None;
        }
        if self.face == Face::Visor(Visor::Grotesque) {
            return Some("Skull Helm");
        }
        ROWS.iter()
            .find(|row| row.name.is_some() && self.fits_row(row, None))
            .and_then(|row| row.name)
    }

    /// The generic name for its shape.
    pub fn generic_name(&self) -> &'static str {
        match self.kind {
            HeadKind::Cap => "Arming Cap",
            HeadKind::Coif => match self.construction {
                Construction::Mail => "Mail Coif",
                Construction::RingMail => "Ring Mail Coif",
                _ => "Coif",
            },
            _ if self.brim.round() => "War Hat",
            _ if self.face.closed() => "Helm",
            _ if self.skull_cap() && self.construction != Construction::Plate => "Cap",
            _ if self.skull_cap() => "Skull Cap",
            _ => "Helmet",
        }
    }

    /// The word for its material, when it goes before the name: anything but iron and
    /// steel on a generic name, and only the stories' metals on a specific one.
    fn material_word(&self, specific: bool) -> Option<&'static str> {
        let word = match self.material {
            Material::Mithril => "Mithril",
            Material::Adamantine => "Adamantine",
            _ if specific => return None,
            Material::Bronze => "Bronze",
            Material::Copper => "Copper",
            Material::Leather | Material::HardenedLeather => "Leather",
            Material::Rattan => "Rattan",
            Material::Horn => "Horn",
            _ if self.construction == Construction::Lamellar => "Lamellar",
            _ => return None,
        };
        // Cloth and mail coifs say what they are already.
        (self.helmet()).then_some(word)
    }

    pub fn name(&self) -> String {
        let specific = self.specific_name();
        let base = specific.unwrap_or(self.generic_name());
        let mut words: Vec<&str> = Vec::new();
        words.extend(self.ornament.prefix());
        words.extend(self.material_word(specific.is_some()));
        words.push(base);
        words.join(" ")
    }
}
