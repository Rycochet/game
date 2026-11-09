//! How fine each material is within its category, and how heavy: first guesses.

use super::Material;

impl Material {
    /// How fine it is within its category, from 1. A first guess that rolls lean on.
    pub fn tier(self) -> u64 {
        match self {
            Self::Amethyst
            | Self::Bone
            | Self::Brass
            | Self::Citrine
            | Self::Coir
            | Self::Copper
            | Self::Cork
            | Self::Cotton
            | Self::Felt
            | Self::Garnet
            | Self::Haircloth
            | Self::Hemp
            | Self::Horn
            | Self::Leather
            | Self::Linen
            | Self::Onyx
            | Self::Peridot
            | Self::Ramie
            | Self::Rattan
            | Self::Rawhide
            | Self::Straw
            | Self::Wood
            | Self::Wool => 1,
            Self::Aquamarine
            | Self::Bronze
            | Self::BuffLeather
            | Self::Chitin
            | Self::Fur
            | Self::HardenedLeather
            | Self::Iron
            | Self::Suede
            | Self::Topaz => 2,
            Self::Emerald
            | Self::Ruby
            | Self::Sapphire
            | Self::Silk
            | Self::Silver
            | Self::Steel => 3,
            Self::Adamantine | Self::Diamond | Self::DragonScale | Self::Gold | Self::Mithril => 4,
        }
    }

    /// How heavy it is, as tenths of a gram for each cubic centimetre. A first guess.
    pub fn density(self) -> u32 {
        match self {
            Self::Cork | Self::Straw => 2,
            Self::Felt => 3,
            Self::Rattan => 5,
            Self::Wood => 7,
            Self::Coir => 12,
            Self::Suede => 8,
            Self::BuffLeather | Self::Fur | Self::Leather => 9,
            Self::Rawhide => 12,
            Self::HardenedLeather => 11,
            Self::Chitin | Self::Horn | Self::Silk | Self::Wool => 13,
            Self::Cotton | Self::Hemp | Self::Linen | Self::Ramie => 15,
            Self::Haircloth => 14,
            Self::Bone => 19,
            Self::DragonScale => 20,
            // Half the weight of steel, as the stories have it.
            Self::Mithril => 39,
            Self::Steel => 78,
            Self::Iron => 79,
            Self::Brass => 85,
            Self::Bronze => 88,
            // Heavier and harder than steel, as the stories have it.
            Self::Adamantine => 95,
            Self::Copper => 90,
            Self::Silver => 105,
            Self::Gold => 193,
            _ => 35,
        }
    }
}
