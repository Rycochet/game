//! What things are made of. A first table for trying this out: the owner means to
//! flesh out materials later, with money values and more categories (fabric, wood).
//! See `docs/context/shared/item.md`.

use serde::{Deserialize, Serialize};

/// What sort of thing a material is, which decides what it can be used for.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Category {
    Bone,
    Gem,
    Metal,
    /// Plant stuff other than timber: rattan, cork.
    Plant,
    Wood,
}

/// A material. Saved by name, so new ones can go anywhere in the list.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Material {
    Amethyst,
    Aquamarine,
    Bone,
    Citrine,
    Copper,
    Cork,
    Diamond,
    Emerald,
    Garnet,
    Gold,
    Iron,
    Peridot,
    Rattan,
    Ruby,
    Sapphire,
    Silver,
    Topaz,
    Wood,
}

impl Material {
    /// Every material, alphabetically by name.
    pub const ALL: [Self; 18] = [
        Self::Amethyst,
        Self::Aquamarine,
        Self::Bone,
        Self::Citrine,
        Self::Copper,
        Self::Cork,
        Self::Diamond,
        Self::Emerald,
        Self::Garnet,
        Self::Gold,
        Self::Iron,
        Self::Peridot,
        Self::Rattan,
        Self::Ruby,
        Self::Sapphire,
        Self::Silver,
        Self::Topaz,
        Self::Wood,
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
            Self::Amethyst => "Amethyst",
            Self::Aquamarine => "Aquamarine",
            Self::Bone => "Bone",
            Self::Citrine => "Citrine",
            Self::Copper => "Copper",
            Self::Cork => "Cork",
            Self::Diamond => "Diamond",
            Self::Emerald => "Emerald",
            Self::Garnet => "Garnet",
            Self::Gold => "Gold",
            Self::Iron => "Iron",
            Self::Peridot => "Peridot",
            Self::Rattan => "Rattan",
            Self::Ruby => "Ruby",
            Self::Sapphire => "Sapphire",
            Self::Silver => "Silver",
            Self::Topaz => "Topaz",
            Self::Wood => "Wood",
        }
    }

    pub fn category(self) -> Category {
        match self {
            Self::Bone => Category::Bone,
            Self::Copper | Self::Gold | Self::Iron | Self::Silver => Category::Metal,
            Self::Cork | Self::Rattan => Category::Plant,
            Self::Wood => Category::Wood,
            _ => Category::Gem,
        }
    }

    /// How fine it is within its category, from 1. A first guess that rolls lean on.
    pub fn tier(self) -> u64 {
        match self {
            Self::Amethyst
            | Self::Bone
            | Self::Citrine
            | Self::Copper
            | Self::Cork
            | Self::Garnet
            | Self::Peridot
            | Self::Rattan
            | Self::Wood => 1,
            Self::Aquamarine | Self::Iron | Self::Topaz => 2,
            Self::Emerald | Self::Ruby | Self::Sapphire | Self::Silver => 3,
            Self::Diamond | Self::Gold => 4,
        }
    }

    /// How heavy it is, as tenths of a gram for each cubic centimetre. A first guess.
    pub fn density(self) -> u32 {
        match self {
            Self::Cork => 2,
            Self::Rattan => 5,
            Self::Wood => 7,
            Self::Bone => 19,
            Self::Iron => 79,
            Self::Copper => 90,
            Self::Silver => 105,
            Self::Gold => 193,
            _ => 35,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_material_is_listed_alphabetically() {
        let names: Vec<_> = Material::ALL
            .iter()
            .map(|material| material.name())
            .collect();
        let mut sorted = names.clone();
        sorted.sort();
        assert_eq!(names, sorted);
    }

    #[test]
    fn the_gems_are_all_the_materials_of_that_category_cheapest_first() {
        let gems = Material::ALL
            .iter()
            .filter(|material| material.category() == Category::Gem);
        assert_eq!(gems.count(), Material::GEMS.len());
        assert!(
            Material::GEMS
                .windows(2)
                .all(|pair| pair[0].category() == Category::Gem && pair[0].tier() <= pair[1].tier())
        );
    }
}
