//! What a pair of foot pieces is called: a specific name when it fits one, otherwise
//! one from the piece and how it is made, with studs and an unusual material first.

use super::{FeetForm, FeetPiece, Toe};
use crate::armour::{Construction, Style};
use crate::material::Material;

impl FeetForm {
    /// A name from history, when it fits one. Armoured boots keep the generic name
    /// players look for (the owner's decision).
    pub fn specific_name(&self) -> Option<&'static str> {
        if self.high {
            return None;
        }
        Some(match (self.piece, self.toe) {
            (FeetPiece::Sabatons, Toe::Claw) => "Clawed Sabatons",
            (FeetPiece::Sabatons, _) if self.construction != Construction::Plate => return None,
            (FeetPiece::Sabatons, Toe::Pointed)
                if self.style == Style::Gothic && self.lames >= 6 =>
            {
                "Gothic Sabatons"
            }
            (FeetPiece::Sabatons, Toe::DuckBill) => "Duck-bill Sabatons",
            (FeetPiece::Sabatons, Toe::BearPaw) => "Bear-paw Sabatons",
            (FeetPiece::Sabatons, Toe::CowMouth) => "Cow-mouth Sabatons",
            (FeetPiece::MailShoes, _) if self.japanese() => "Kusari Tabi",
            (FeetPiece::FootGuards, _) if self.japanese() => "Kogake",
            _ => return None,
        })
    }

    /// The generic name for its piece and construction.
    pub fn generic_name(&self) -> &'static str {
        use Construction as C;
        match (self.piece, self.construction) {
            (FeetPiece::Sabatons, C::Leather) if self.high => "Leather Armoured Boots",
            (FeetPiece::Sabatons, _) if self.high => "Armoured Boots",
            (FeetPiece::Sabatons, C::Leather) => "Leather Sabatons",
            (FeetPiece::Sabatons, C::Scale) => "Scale Sabatons",
            (FeetPiece::Sabatons, C::Splint) => "Splinted Sabatons",
            (FeetPiece::Sabatons, _) => "Sabatons",
            (FeetPiece::MailShoes, C::RingMail) => "Ring Mail Shoes",
            (FeetPiece::MailShoes, _) => "Mail Shoes",
            (FeetPiece::FootGuards, C::Leather) => "Leather Foot Guards",
            (FeetPiece::FootGuards, C::MailAndPlate) => "Plated Foot Guards",
            (FeetPiece::FootGuards, _) => "Foot Guards",
        }
    }

    /// The word for its material, when it goes before a name: anything but iron,
    /// steel, leather, cloth, and the bronze every Classical foot is made of.
    fn material_word(&self) -> Option<&'static str> {
        match self.material {
            Material::Iron | Material::Steel | Material::Leather | Material::HardenedLeather => {
                None
            }
            Material::Bronze if self.style == Style::Classical => None,
            material => Some(material.name()),
        }
    }

    /// Its name, with studs and an unusual material first.
    pub fn name(&self) -> String {
        let mut words = Vec::new();
        if self.studded {
            words.push("Studded");
        }
        if let Some(material) = self.material_word() {
            words.push(material);
        }
        let name = self.specific_name().unwrap_or_else(|| {
            let generic = self.generic_name();
            // Dragon scale sabatons, not dragon scale scale sabatons.
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
