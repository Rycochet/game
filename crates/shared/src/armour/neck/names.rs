//! What a neck piece is called: a specific name when it fits one, otherwise one from
//! its kind and how it is made, with spikes, studs and an unusual material first.

use super::{NeckForm, NeckKind, Plates};
use crate::armour::{Construction, Style};
use crate::material::Material;

impl NeckForm {
    /// A name from history, when it fits one.
    pub fn specific_name(&self) -> Option<&'static str> {
        use Construction as C;
        Some(match (self.kind, self.construction) {
            (NeckKind::Collar, C::Mail) => "Standard",
            (NeckKind::Collar, C::PlatesInCloth) => "Guruwa",
            (NeckKind::Gorget, C::Plate) if self.plates == Plates::Shoulders => "Colletin",
            (NeckKind::Gorget, C::Laminar | C::Lamellar) if self.japanese() => "Nodowa",
            (NeckKind::Mantle, C::Mail) => "Bishop's Mantle",
            _ => return None,
        })
    }

    /// The generic name for its kind and construction.
    pub fn generic_name(&self) -> &'static str {
        use Construction as C;
        let collar = self.plates == Plates::Collar;
        match (self.kind, self.construction) {
            (NeckKind::Collar, C::Padded) => "Padded Collar",
            (NeckKind::Collar, C::RingMail) => "Ring Mail Collar",
            (NeckKind::Collar, C::PlatesInCloth) => "Plated Collar",
            (NeckKind::Collar, _) => "Mail Collar",
            (NeckKind::Gorget, C::Leather) if collar => "Leather Collar",
            (NeckKind::Gorget, C::Leather) => "Leather Gorget",
            (NeckKind::Gorget, C::Scale) if collar => "Scale Collar",
            (NeckKind::Gorget, C::Scale) => "Scale Gorget",
            (NeckKind::Gorget, C::Lamellar) => "Lamellar Gorget",
            (NeckKind::Gorget, C::Laminar) => "Laminar Gorget",
            (NeckKind::Gorget, _) if collar => "Plate Collar",
            (NeckKind::Gorget, _) => "Gorget",
            (NeckKind::Bevor, C::Leather) => "Leather Bevor",
            (NeckKind::Bevor, _) => "Bevor",
            (NeckKind::Aventail, C::RingMail) => "Ring Mail Aventail",
            (NeckKind::Aventail, C::Lamellar) => "Lamellar Aventail",
            (NeckKind::Aventail, C::Scale) => "Scale Aventail",
            (NeckKind::Aventail, C::Padded) => "Quilted Aventail",
            (NeckKind::Aventail, C::Leather) => "Leather Aventail",
            (NeckKind::Aventail, C::MailAndPlate) => "Plated Aventail",
            (NeckKind::Aventail, _) => "Aventail",
            (NeckKind::Mantle, C::Scale) => "Scale Mantle",
            (NeckKind::Mantle, _) => "Mail Mantle",
        }
    }

    /// The word for its material, when it goes before a name: anything but iron,
    /// steel, leather, cloth, and the bronze Classical plate is made of.
    fn material_word(&self) -> Option<&'static str> {
        match self.material {
            Material::Iron
            | Material::Steel
            | Material::Leather
            | Material::HardenedLeather
            | Material::Linen
            | Material::Cotton
            | Material::Wool => None,
            Material::Bronze if self.style == Style::Classical => None,
            material => Some(material.name()),
        }
    }

    /// Its name, with spikes, studs and an unusual material first.
    pub fn name(&self) -> String {
        let mut words = Vec::new();
        if self.spikes {
            words.push("Spiked");
        }
        if self.studded {
            words.push("Studded");
        }
        if let Some(material) = self.material_word() {
            words.push(material);
        }
        let name = self.specific_name().unwrap_or_else(|| {
            let generic = self.generic_name();
            // Dragon scale gorgets, not dragon scale scale gorgets.
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
