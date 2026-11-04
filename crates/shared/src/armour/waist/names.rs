//! What a waist piece is called: a specific name when it fits one, otherwise one from
//! its shape and how it is made, with studs and an unusual material first.

use super::{Length, WaistForm, WaistKind, WaistPiece};
use crate::armour::{Construction, Style};
use crate::material::Material;

impl WaistForm {
    /// A name from history, when it fits one.
    pub fn specific_name(&self) -> Option<&'static str> {
        use Construction as C;
        use WaistPiece as P;
        Some(match (self.piece, self.construction, self.style) {
            _ if self.kind == WaistKind::Cuirassier => "Cuirassier Tassets",
            (P::Skirt, C::Plate, Style::Maximilian | Style::Greenwich) => "Steel Bases",
            (P::Skirt, C::Plate, _) => "Tonlet",
            (P::Tassets, C::Plate, Style::Classical) => "Dendra Hip Plates",
            (P::Tassets, C::Plate, _) if self.lames >= 3 => "Almain Tassets",
            (P::Panels, _, Style::Japanese) => "Kusazuri",
            (P::Pteruges, _, _) if self.front => "Cingulum Apron",
            (P::Pteruges, _, Style::Classical) => "Pteruges",
            (P::Pteruges, C::Leather, _) if self.studded && self.length == Length::Thigh => {
                "War Kilt"
            }
            (P::Skirt, C::Padded | C::Lamellar, Style::Byzantine) => "Kremasmata",
            (P::Girdle, C::Plate, Style::Classical) if self.front => "Mitra",
            (P::Girdle, C::Plate, Style::Classical) => "Italic Belt",
            _ => return None,
        })
    }

    /// The generic name for its shape and construction.
    pub fn generic_name(&self) -> &'static str {
        use Construction as C;
        use WaistPiece as P;
        match (self.piece, self.construction) {
            (P::Girdle, _) => "War Girdle",
            (P::Fauld, C::Plate) => "Fauld",
            (P::Fauld, C::Laminar) => "Laminar Fauld",
            (P::Fauld, _) => "Leather Fauld",
            (P::Tassets, C::Plate) => "Tassets",
            (P::Tassets, C::Leather) => "Leather Tassets",
            (P::Tassets, C::Splint) => "Splinted Tassets",
            (P::Tassets, C::PlatesInCloth) => "Brigandine Tassets",
            (P::Tassets, C::MailAndPlate) => "Plated Tassets",
            (P::Tassets, _) => "Laminar Tassets",
            (P::Panels, C::Scale) => "Scale Hip Panels",
            (P::Panels, C::Laminar) => "Laminar Hip Panels",
            (P::Panels, C::Leather) => "Leather Hip Panels",
            (P::Panels, C::PlatesInCloth) => "Brigandine Hip Panels",
            (P::Panels, _) => "Lamellar Hip Panels",
            (P::Pteruges, _) => "Pteruges",
            (P::Breeches, _) => "Mail Breeches",
            (P::Skirt, C::Plate) => "Plate Skirt",
            (P::Skirt, C::Mail) => "Mail Skirt",
            (P::Skirt, C::RingMail) => "Ring Mail Skirt",
            (P::Skirt, C::Scale) => "Scale Skirt",
            (P::Skirt, C::Lamellar) => "Lamellar Skirt",
            (P::Skirt, C::Laminar) => "Laminar Skirt",
            (P::Skirt, C::PlatesInCloth) => "Brigandine Skirt",
            (P::Skirt, C::MailAndPlate) => "Plated Skirt",
            (P::Skirt, C::Leather) => "Leather Skirt",
            (P::Skirt, _) => "Padded Skirt",
        }
    }

    /// The word for its material, when it goes before a name: anything but iron,
    /// steel, leather, cloth, and the bronze every Classical plate is made of.
    fn material_word(&self) -> Option<&'static str> {
        match self.material {
            Material::Iron
            | Material::Steel
            | Material::Leather
            | Material::HardenedLeather
            | Material::Linen
            | Material::Cotton
            | Material::Wool
            | Material::Silk => None,
            Material::Bronze if self.style == Style::Classical => None,
            material => Some(material.name()),
        }
    }

    /// Its name, with studs and an unusual material first.
    pub fn name(&self) -> String {
        let mut words = Vec::new();
        if self.studded && self.specific_name() != Some("War Kilt") {
            words.push("Studded");
        }
        if let Some(material) = self.material_word() {
            words.push(material);
        }
        let name = self.specific_name().unwrap_or_else(|| {
            let generic = self.generic_name();
            // Dragon scale skirts, not dragon scale scale skirts.
            if self.material == Material::DragonScale {
                generic.trim_start_matches("Scale ")
            } else {
                generic
            }
        });
        words.push(name);
        words.join(" ")
    }
}
