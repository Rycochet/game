//! What a set is called: by its kind, its torso's construction and its style.

use super::{SetForm, SetKind};
use crate::armour::{Construction, Style};
use crate::material::Material;

impl SetForm {
    /// Its name, with an unusual material first ("Mithril Hauberk").
    pub fn name(&self) -> String {
        let name = self.base_name();
        match unusual(self.material()) {
            Some(material) => format!("{material} {name}"),
            None => name,
        }
    }

    fn base_name(&self) -> String {
        let construction = self.construction();
        let style = self.style();
        let noun = noun(construction);
        match self.kind {
            SetKind::Coat => match construction {
                Construction::Mail => "Hauberk".to_owned(),
                Construction::Leather if self.material() == Material::BuffLeather => {
                    "Buff Coat".to_owned()
                }
                _ => match self.torso.specific_name() {
                    Some(name) => name.to_owned(),
                    None => format!("{noun} Coat"),
                },
            },
            SetKind::Corslet => "Corslet".to_owned(),
            SetKind::Half if construction == Construction::Plate => "Half Armour".to_owned(),
            SetKind::Half => format!("{noun} Half Armour"),
            SetKind::Full => match (construction, style) {
                (_, Style::Japanese) => "Gusoku".to_owned(),
                (Construction::Plate, style) if style.european() => {
                    format!("{} Harness", style.name())
                }
                (Construction::Plate, Style::Elven | Style::Dwarven) => {
                    format!("{} Plate", style.name())
                }
                (Construction::Mail, _) => "Full Mail".to_owned(),
                _ => format!("Full {noun} Armour"),
            },
        }
    }
}

/// How a construction reads in a set's name.
fn noun(construction: Construction) -> &'static str {
    match construction {
        Construction::Padded => "Padded",
        Construction::Leather => "Leather",
        Construction::Mail => "Mail",
        Construction::Scale => "Scale",
        Construction::Lamellar => "Lamellar",
        Construction::Laminar => "Laminar",
        Construction::PlatesInCloth => "Brigandine",
        Construction::MailAndPlate => "Mail and Plate",
        Construction::Splint => "Splint",
        Construction::Plate => "Plate",
        Construction::Mirror => "Mirror",
        Construction::RingMail => "Ring Mail",
        Construction::Woven => "Rattan",
    }
}

/// Materials worth naming first.
fn unusual(material: Material) -> Option<&'static str> {
    match material {
        Material::Mithril
        | Material::Adamantine
        | Material::DragonScale
        | Material::Chitin
        | Material::Bronze => Some(material.name()),
        _ => None,
    }
}
