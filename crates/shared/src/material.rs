//! What things are made of. A first table for trying this out: the owner means to
//! flesh out materials later, with money values and more categories (fabric, wood).
//! See `docs/context/shared/item.md`.

use serde::{Deserialize, Serialize};

/// What sort of thing a material is, which decides what it can be used for.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Category {
    Bone,
    Gem,
    /// Hides: soft, hardened (cuir bouilli) or thick buff.
    Leather,
    Metal,
    /// Plant stuff other than timber: rattan, cork.
    Plant,
    /// Cloth and the fibres it is woven from.
    Textile,
    Wood,
}

/// A material. Saved by name, so new ones can go anywhere in the list.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Material {
    Adamantine,
    Amethyst,
    Aquamarine,
    Bone,
    Brass,
    Bronze,
    BuffLeather,
    Chitin,
    Citrine,
    Coir,
    Copper,
    Cork,
    Cotton,
    Diamond,
    DragonScale,
    Emerald,
    Felt,
    Fur,
    Garnet,
    Gold,
    HardenedLeather,
    Horn,
    Iron,
    Leather,
    Linen,
    Mithril,
    Peridot,
    Rattan,
    Rawhide,
    Ruby,
    Sapphire,
    Silk,
    Silver,
    Steel,
    Straw,
    Suede,
    Topaz,
    Wood,
    Wool,
}

impl Material {
    /// Every material, alphabetically by name.
    pub const ALL: [Self; 39] = [
        Self::Adamantine,
        Self::Amethyst,
        Self::Aquamarine,
        Self::Bone,
        Self::Brass,
        Self::Bronze,
        Self::BuffLeather,
        Self::Chitin,
        Self::Citrine,
        Self::Coir,
        Self::Copper,
        Self::Cork,
        Self::Cotton,
        Self::Diamond,
        Self::DragonScale,
        Self::Emerald,
        Self::Felt,
        Self::Fur,
        Self::Garnet,
        Self::Gold,
        Self::HardenedLeather,
        Self::Horn,
        Self::Iron,
        Self::Leather,
        Self::Linen,
        Self::Mithril,
        Self::Peridot,
        Self::Rattan,
        Self::Rawhide,
        Self::Ruby,
        Self::Sapphire,
        Self::Silk,
        Self::Silver,
        Self::Steel,
        Self::Straw,
        Self::Suede,
        Self::Topaz,
        Self::Wood,
        Self::Wool,
    ];

    /// Every gem, cheapest first and alphabetically within a tier.
    pub const GEMS: [Self; 10] = [
        Self::Amethyst,
        Self::Citrine,
        Self::Garnet,
        Self::Peridot,
        Self::Aquamarine,
        Self::Topaz,
        Self::Emerald,
        Self::Ruby,
        Self::Sapphire,
        Self::Diamond,
    ];

    pub fn name(self) -> &'static str {
        match self {
            Self::Adamantine => "Adamantine",
            Self::Amethyst => "Amethyst",
            Self::Aquamarine => "Aquamarine",
            Self::Bone => "Bone",
            Self::Brass => "Brass",
            Self::Bronze => "Bronze",
            Self::BuffLeather => "Buff Leather",
            Self::Chitin => "Chitin",
            Self::Citrine => "Citrine",
            Self::Coir => "Coir",
            Self::Copper => "Copper",
            Self::Cork => "Cork",
            Self::Cotton => "Cotton",
            Self::Diamond => "Diamond",
            Self::DragonScale => "Dragon Scale",
            Self::Emerald => "Emerald",
            Self::Felt => "Felt",
            Self::Fur => "Fur",
            Self::Garnet => "Garnet",
            Self::Gold => "Gold",
            Self::HardenedLeather => "Hardened Leather",
            Self::Horn => "Horn",
            Self::Iron => "Iron",
            Self::Leather => "Leather",
            Self::Linen => "Linen",
            Self::Mithril => "Mithril",
            Self::Peridot => "Peridot",
            Self::Rattan => "Rattan",
            Self::Rawhide => "Rawhide",
            Self::Ruby => "Ruby",
            Self::Sapphire => "Sapphire",
            Self::Silk => "Silk",
            Self::Silver => "Silver",
            Self::Steel => "Steel",
            Self::Straw => "Straw",
            Self::Suede => "Suede",
            Self::Topaz => "Topaz",
            Self::Wood => "Wood",
            Self::Wool => "Wool",
        }
    }

    pub fn category(self) -> Category {
        match self {
            Self::Bone | Self::Chitin | Self::DragonScale | Self::Horn => Category::Bone,
            Self::BuffLeather
            | Self::Fur
            | Self::HardenedLeather
            | Self::Leather
            | Self::Rawhide
            | Self::Suede => Category::Leather,
            Self::Adamantine
            | Self::Brass
            | Self::Bronze
            | Self::Copper
            | Self::Gold
            | Self::Iron
            | Self::Mithril
            | Self::Silver
            | Self::Steel => Category::Metal,
            Self::Coir | Self::Cork | Self::Rattan | Self::Straw => Category::Plant,
            Self::Cotton | Self::Felt | Self::Linen | Self::Silk | Self::Wool => Category::Textile,
            Self::Wood => Category::Wood,
            _ => Category::Gem,
        }
    }

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
            | Self::Horn
            | Self::Leather
            | Self::Linen
            | Self::Peridot
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
            Self::Cotton | Self::Linen => 15,
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

#[cfg(test)]
mod tests;
