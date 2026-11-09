//! What a piece of underclothing is called: the garment's name from the research's
//! signatures, an unusual material or lawn in front ("Silk Chemise", "Lawn Shirt"),
//! and any stays after it ("Chemise and Stays").

use super::cloth::materials;
use super::{Cloth, Decoration, Garment, Stays, UnderForm};
use crate::material::Material;

impl UnderForm {
    /// The garment's own name, without its material.
    pub fn specific_name(&self) -> &'static str {
        let material = self.material;
        match &self.garment {
            Garment::Shirt(_) if material == Material::Haircloth => "Hair Shirt",
            Garment::Shirt(shirt) => shirt.name(),
            Garment::Braies(braies) => braies.name(),
            Garment::Band(band) => band.name(material),
            Garment::Loincloth(loin) => loin.name(material),
            Garment::Hose(hose) => hose.name(),
            Garment::Socks(socks) => socks.name(),
            Garment::Petticoat(skirt) => skirt.name(),
            Garment::Coif(coif) => coif.name(self.decoration == Decoration::GoldThread),
            Garment::Wimple(wimple) => wimple.name(),
        }
    }

    /// Its full name.
    pub fn name(&self) -> String {
        let specific = self.specific_name();
        let mut name = match self.material_word() {
            Some(word) => format!("{word} {specific}"),
            None => specific.to_owned(),
        };
        if let Garment::Shirt(shirt) = &self.garment {
            match shirt.stays {
                Stays::None => {}
                Stays::PairOfBodies => name.push_str(" and Pair of Bodies"),
                Stays::Stays => name.push_str(" and Stays"),
            }
        }
        name
    }

    /// The word for its cloth, when it is worth saying: lawn always, a material other
    /// than its garment's usual one, and a coif's always (the mail Coif is armour).
    fn material_word(&self) -> Option<&'static str> {
        if self.cloth == Cloth::Lawn {
            return Some("Lawn");
        }
        let in_name = matches!(self.material, Material::Haircloth | Material::Felt);
        let usual = materials(&self.garment)
            .first()
            .is_some_and(|&(first, _)| first == self.material);
        // Leather is worth saying even where it is the only choice ("Leather Leggings").
        let leather = self.material == Material::Leather;
        let coif = self.specific_name() == "Coif";
        (coif || leather || !(usual || in_name)).then(|| self.material.name())
    }
}
