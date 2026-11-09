//! What underclothes weigh and take up. Weights are the research's estimates for plain
//! linen (woven wool for hose), from cloth areas, changed by the material and what is
//! added: stays, a sole, a codpiece. No article gave one.

use super::{
    BraiesLength, Cloth, Garment, HoseHeight, Length, SkirtFullness, SkirtLength, SockHeight,
    Stays, UnderForm,
};
use crate::creature::Size;
use crate::item::Shape;
use crate::material::Material;

impl UnderForm {
    fn base(&self) -> u32 {
        match &self.garment {
            Garment::Shirt(shirt) => shirt.base_grams(),
            Garment::Braies(braies) => braies.base_grams(),
            Garment::Band(band) => band.base_grams(),
            Garment::Loincloth(loin) => loin.base_grams(),
            Garment::Hose(hose) => hose.base_grams(),
            Garment::Socks(socks) => socks.base_grams(),
            Garment::Petticoat(skirt) => skirt.base_grams(),
            Garment::Coif(coif) => coif.base_grams(),
            Garment::Wimple(wimple) => wimple.base_grams(),
        }
    }

    /// Hundredths of the usual weight its cloth makes it. Hose and socks are measured in
    /// wool, everything else in linen.
    fn factor(&self) -> u32 {
        let woollen = matches!(self.garment, Garment::Hose(_) | Garment::Socks(_));
        let factor = match self.material {
            Material::Linen | Material::Ramie => 100,
            Material::Hemp => 110,
            Material::Cotton => 90,
            Material::Silk => 60,
            Material::Wool => 130,
            Material::Haircloth => 160,
            Material::Leather => 170,
            Material::Felt => 220,
            _ => 100,
        };
        let factor = if woollen { factor * 100 / 130 } else { factor };
        match self.cloth {
            Cloth::Lawn => factor * 70 / 100,
            _ => factor,
        }
    }

    fn extras(&self) -> u32 {
        match &self.garment {
            Garment::Shirt(shirt) => shirt.stays_grams(),
            Garment::Hose(hose) => hose.extra_grams(),
            _ => 0,
        }
    }

    /// What it weighs at an ordinary size, in grams.
    pub fn grams(&self) -> u32 {
        (self.base() * self.factor() / 100 + self.extras()).max(5)
    }

    /// What it weighs made at this size: bigger across, as thick, so by the square of
    /// its size (as footwear).
    pub fn grams_at(&self, size: Size) -> u32 {
        let quarters = size.quarters() as u64;
        (self.grams() as u64 * quarters * quarters / 16).max(1) as u32
    }

    /// The cells it takes up at an ordinary size, wide and long.
    pub fn shape(&self) -> Shape {
        let (width, height) = match &self.garment {
            Garment::Shirt(shirt) if shirt.stays != Stays::None => (2, 3),
            Garment::Shirt(shirt) if shirt.length >= Length::Knee => (2, 3),
            Garment::Shirt(_) => (2, 2),
            Garment::Braies(braies) if braies.length == BraiesLength::Thigh => (2, 1),
            Garment::Braies(_) => (2, 2),
            Garment::Band(_) | Garment::Loincloth(_) => (2, 1),
            Garment::Hose(hose) if hose.height == HoseHeight::Waist => (2, 3),
            Garment::Hose(_) => (2, 2),
            Garment::Socks(socks) if socks.height >= SockHeight::Calf => (1, 2),
            Garment::Socks(_) => (1, 1),
            Garment::Petticoat(skirt)
                if skirt.length == SkirtLength::Ankle
                    && skirt.fullness == SkirtFullness::VeryFull =>
            {
                (2, 3)
            }
            Garment::Petticoat(_) => (2, 2),
            Garment::Coif(_) | Garment::Wimple(_) => (1, 1),
        };
        Shape::rectangle(width, height)
    }
}
