//! What a leg piece is called: a specific name when it fits one, otherwise one from
//! how much of the leg it covers and how it is made, with ornaments, studs and an
//! unusual material first.

use super::{Cover, LegsForm, LegsKind, Ornament, Reinforce};
use crate::armour::arms::Sides;
use crate::armour::{Construction, Style};
use crate::material::Material;

impl LegsForm {
    /// A name from history, when it fits one (the owner kept these).
    pub fn specific_name(&self) -> Option<&'static str> {
        use Construction as C;
        let both = self.sides == Sides::Both;
        let thigh = matches!(self.cover, Cover::Thigh | Cover::ThighAndKnee);
        Some(match (self.construction, self.style, self.cover) {
            _ if self.kind == LegsKind::Padded => "Padded Chausses",
            (C::Mail, _, Cover::Whole) if self.reinforce != Reinforce::None => {
                "Reinforced Chausses"
            }
            (C::Mail, _, Cover::Whole) if !self.closed => "Demi-chausses",
            (C::Mail, _, Cover::Whole) if self.footed() => "Chausses",
            (C::Mail, _, Cover::Whole) => "Footless Chausses",
            (C::Plate, Style::Japanese, _) => "Tsutsu-suneate",
            (_, Style::Japanese, _) if thigh => "Haidate",
            (_, Style::Japanese, _) => "Suneate",
            (C::Plate, Style::Classical, Cover::KneeAndShin) => {
                if both {
                    "Knemides"
                } else {
                    "Knemis"
                }
            }
            (C::Plate, Style::Classical, Cover::Shin) if self.coiled() => "Ring Greaves",
            (C::Plate, Style::Classical, Cover::Shin) => {
                if both {
                    "Ocreae"
                } else {
                    "Ocrea"
                }
            }
            (C::Plate, Style::Classical, Cover::Thigh) => "Perimeridia",
            (_, _, Cover::Ankle) => "Episphyria",
            (C::Laminar, Style::Classical, _) => "Leg Manicae",
            (C::Scale, Style::Classical, _) => "Demi-chausses",
            (C::MailAndPlate, Style::IndoPersian, Cover::ThighAndKnee) => "Dizçek",
            (C::Plate, _, Cover::Shin) if !self.closed => "Schynbalds",
            (C::Padded, _, _) => "Gamboised Cuisses",
            _ => return None,
        })
    }

    /// The generic name for its cover and construction.
    pub fn generic_name(&self) -> &'static str {
        use Construction as C;
        use Cover as V;
        match (self.cover, self.construction) {
            (V::Knee, C::Plate) => "Poleyns",
            (V::Knee, C::Leather) => "Leather Knee Guards",
            (V::Knee, _) => "Knee Guards",
            (V::Ankle, _) => "Ankle Guards",
            (V::Thigh | V::ThighAndKnee, C::Plate) => "Cuisses",
            (V::Thigh | V::ThighAndKnee, C::Leather) => "Leather Cuisses",
            (V::Thigh | V::ThighAndKnee, C::Splint) => "Splinted Cuisses",
            (V::Thigh | V::ThighAndKnee, C::PlatesInCloth) => "Brigandine Cuisses",
            (V::Thigh | V::ThighAndKnee, C::MailAndPlate) => "Plated Cuisses",
            (V::Thigh | V::ThighAndKnee, C::Mail) => "Mail Thigh Guards",
            (V::Thigh | V::ThighAndKnee, C::RingMail) => "Ring Mail Thigh Guards",
            (V::Thigh | V::ThighAndKnee, C::Scale) => "Scale Thigh Guards",
            (V::Thigh | V::ThighAndKnee, C::Lamellar) => "Lamellar Thigh Guards",
            (V::Thigh | V::ThighAndKnee, _) => "Thigh Guards",
            (V::Whole, C::Plate) => "Leg Harness",
            (V::Whole, C::Leather) => "Leather Leg Harness",
            (V::Whole, C::Splint) => "Splinted Legs",
            (V::Whole, C::Mail) => "Mail Chausses",
            (V::Whole, C::RingMail) => "Ring Mail Leggings",
            (V::Whole, C::Scale) => "Scale Leggings",
            (V::Whole, C::Lamellar) => "Lamellar Leggings",
            (V::Whole, C::Laminar) => "Laminar Leggings",
            (V::Whole, C::PlatesInCloth) => "Brigandine Leggings",
            (V::Whole, C::MailAndPlate) => "Plated Leggings",
            (V::Whole, _) => "Leg Guards",
            (_, C::Plate) if !self.closed => "Demi-greaves",
            (_, C::Plate) => "Greaves",
            (_, C::Leather) => "Leather Greaves",
            (_, C::Splint) => "Splinted Greaves",
            (_, C::Mail) => "Mail Greaves",
            (_, C::Scale) => "Scale Greaves",
            (_, C::Lamellar) => "Lamellar Greaves",
            (_, C::Laminar) => "Hooped Greaves",
            (_, C::PlatesInCloth) => "Brigandine Greaves",
            (_, C::MailAndPlate) => "Plated Greaves",
            _ => "Greaves",
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

    /// Its name, with ornaments, studs and an unusual material first.
    pub fn name(&self) -> String {
        let mut words = Vec::new();
        match self.ornament {
            Ornament::None => {}
            Ornament::KneeSpikes => words.push("Spiked"),
            Ornament::ShinFins => words.push("Bladed"),
        }
        if self.studded {
            words.push("Studded");
        }
        if let Some(material) = self.material_word() {
            words.push(material);
        }
        let name = self.specific_name().unwrap_or_else(|| {
            let generic = self.generic_name();
            // Dragon scale greaves, not dragon scale scale greaves.
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
