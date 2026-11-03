//! What an arm piece is called: a specific name when it fits one, otherwise one from
//! how much of the arm it covers and how it is made, with spikes, studs and an unusual
//! material first.

use super::{ArmsForm, Reach, Sides, Spikes};
use crate::armour::{Construction, Style};
use crate::material::Material;

impl ArmsForm {
    /// A name from history, when it fits one (the owner kept these).
    pub fn specific_name(&self) -> Option<&'static str> {
        let both = self.sides == Sides::Both;
        if self.kote() {
            Some(if self.reach == Reach::Whole {
                "Kote"
            } else {
                "Han Kote"
            })
        } else if self.style == Style::Japanese && self.construction == Construction::Mail {
            Some("Kusari Gote")
        } else if self.manica() {
            Some(if both { "Manicae" } else { "Manica" })
        } else if self.bazubands() {
            Some("Bazubands")
        } else if self.construction == Construction::Leather && !both {
            Some("Archer's Bracer")
        } else {
            None
        }
    }

    /// The generic name for its reach and construction.
    pub fn generic_name(&self) -> &'static str {
        use Construction as C;
        match (self.reach, self.construction) {
            (Reach::Whole, C::Plate) => "Arm Harness",
            (Reach::Whole, C::Leather) => "Leather Arm Harness",
            (Reach::Whole, C::Splint) => "Splinted Arms",
            (Reach::Whole, C::Mail) => "Mail Sleeves",
            (Reach::Whole, C::RingMail) => "Ring Mail Sleeves",
            (Reach::Whole, C::Scale) => "Scale Sleeves",
            (Reach::Whole, C::Lamellar) => "Lamellar Sleeves",
            (Reach::Whole, C::Padded) => "Padded Sleeves",
            (Reach::Whole, C::PlatesInCloth) => "Brigandine Sleeves",
            (Reach::Whole, _) => "Armoured Sleeves",
            (Reach::Wrist, C::Leather) => "Leather Bracers",
            (Reach::Wrist, _) => "Bracers",
            (Reach::Elbow, C::Leather) => "Leather Couters",
            (Reach::Elbow, _) => "Couters",
            (Reach::UpperAndElbow, C::Leather) => "Leather Rerebraces",
            (Reach::UpperAndElbow, C::Splint) => "Splinted Rerebraces",
            (Reach::UpperAndElbow, _) => "Rerebraces",
            (_, C::Leather) => "Leather Vambraces",
            (_, C::Splint) => "Splinted Vambraces",
            (_, C::Scale) => "Scale Vambraces",
            (_, C::Lamellar) => "Lamellar Vambraces",
            (_, C::Laminar) => "Laminar Vambraces",
            (_, C::Padded) => "Quilted Vambraces",
            (_, C::PlatesInCloth) => "Brigandine Vambraces",
            (_, C::MailAndPlate) => "Plated Vambraces",
            _ => "Vambraces",
        }
    }

    /// The word for its material, when it goes before a generic name: anything but
    /// iron, steel, leather and cloth.
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
            material => Some(material.name()),
        }
    }

    /// Its name, with spikes, studs and an unusual material first.
    pub fn name(&self) -> String {
        let mut words = Vec::new();
        if self.spikes != Spikes::None {
            words.push("Spiked");
        }
        if self.studded {
            words.push("Studded");
        }
        let generic = match self.specific_name() {
            Some(name) => name,
            None => {
                if let Some(material) = self.material_word() {
                    words.push(material);
                }
                // Dragon scale sleeves, not dragon scale scale sleeves.
                let generic = self.generic_name();
                match self.construction {
                    Construction::Scale if self.material == Material::DragonScale => {
                        generic.trim_start_matches("Scale ")
                    }
                    _ => generic,
                }
            }
        };
        words.push(generic);
        words.join(" ")
    }
}
