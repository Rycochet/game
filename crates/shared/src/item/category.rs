//! `ItemCategory`: the broad sorts of item, for filtering and sorting grids.

use serde::{Deserialize, Serialize};

use super::ItemKind;

/// A broad sort of item. Every kind is in exactly one.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub enum ItemCategory {
    Weapons,
    Armour,
    Clothing,
    Jewellery,
    Containers,
    Food,
    Valuables,
    Materials,
    Other,
}

impl ItemCategory {
    /// Every category, in the order they are listed.
    pub const ALL: [Self; 9] = [
        Self::Weapons,
        Self::Armour,
        Self::Clothing,
        Self::Jewellery,
        Self::Containers,
        Self::Food,
        Self::Valuables,
        Self::Materials,
        Self::Other,
    ];

    pub const fn name(self) -> &'static str {
        match self {
            Self::Weapons => "Weapons",
            Self::Armour => "Armour",
            Self::Clothing => "Clothing",
            Self::Jewellery => "Jewellery",
            Self::Containers => "Containers",
            Self::Food => "Food",
            Self::Valuables => "Valuables",
            Self::Materials => "Materials",
            Self::Other => "Other",
        }
    }
}

impl ItemKind {
    /// Its category. Every kind is listed by hand, with no catch-all, so a new kind
    /// can't build until it has one.
    pub const fn category(self) -> ItemCategory {
        use ItemCategory::*;
        match self {
            Self::Axe
            | Self::Boomerang
            | Self::Dagger
            | Self::Knife
            | Self::Longsword
            | Self::Polearm
            | Self::ShortSword
            | Self::Spear
            | Self::Sword => Weapons,
            Self::ArmArmour
            | Self::ArmingCap
            | Self::ArmingVest
            | Self::Barding
            | Self::BevoredHelm
            | Self::Chausses
            | Self::FootArmour
            | Self::Coif
            | Self::Collar
            | Self::Gorget
            | Self::Bevor
            | Self::Aventail
            | Self::Mantle
            | Self::DemiGauntlets
            | Self::FullArmour
            | Self::Gambeson
            | Self::Gauntlets
            | Self::Helmet
            | Self::LegArmour
            | Self::PaddedChausses
            | Self::PaddedGloves
            | Self::ShoulderArmour
            | Self::TorsoArmour
            | Self::CuirassierTassets
            | Self::WaistArmour => Armour,
            Self::Belt
            | Self::Boots
            | Self::Caparison
            | Self::Cloak
            | Self::FingerlessGloves
            | Self::Gloves
            | Self::Hat
            | Self::Shoes
            | Self::Socks
            | Self::Surcoat
            | Self::TShirt
            | Self::Tabard
            | Self::Trousers => Clothing,
            Self::Necklace | Self::Ring => Jewellery,
            Self::Backpack | Self::Pouch | Self::SmallBox => Containers,
            Self::Apple => Food,
            Self::Coin | Self::Gem => Valuables,
            Self::Stone => Materials,
            Self::Lantern => Other,
        }
    }
}

#[cfg(test)]
mod tests;
