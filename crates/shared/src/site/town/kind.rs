//! What a building is, and the odds of each kind by town size.

use serde::{Deserialize, Serialize};

/// What a building is. Decides its door's picture; later what it trades in.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum BuildingKind {
    House,
    Shrine,
    Inn,
    Tavern,
    Blacksmith,
    Herbalist,
    Carpenter,
    Tailor,
    Leatherworker,
    Fletcher,
    Church,
    Weaponsmith,
    Armoursmith,
    PotionShop,
    TrainingHall,
    ScrollShop,
    AdventurersGuild,
    Jeweller,
    Library,
    Bank,
    GemCutter,
    BlackMarket,
    Cathedral,
}

impl BuildingKind {
    pub const ALL: [Self; 23] = [
        Self::House,
        Self::Shrine,
        Self::Inn,
        Self::Tavern,
        Self::Blacksmith,
        Self::Herbalist,
        Self::Carpenter,
        Self::Tailor,
        Self::Leatherworker,
        Self::Fletcher,
        Self::Church,
        Self::Weaponsmith,
        Self::Armoursmith,
        Self::PotionShop,
        Self::TrainingHall,
        Self::ScrollShop,
        Self::AdventurersGuild,
        Self::Jeweller,
        Self::Library,
        Self::Bank,
        Self::GemCutter,
        Self::BlackMarket,
        Self::Cathedral,
    ];

    /// What players call it.
    pub fn name(self) -> &'static str {
        match self {
            Self::House => "House",
            Self::Shrine => "Shrine",
            Self::Inn => "Inn",
            Self::Tavern => "Tavern",
            Self::Blacksmith => "Blacksmith",
            Self::Herbalist => "Herbalist",
            Self::Carpenter => "Carpenter",
            Self::Tailor => "Tailor",
            Self::Leatherworker => "Leatherworker",
            Self::Fletcher => "Fletcher",
            Self::Church => "Church",
            Self::Weaponsmith => "Weaponsmith",
            Self::Armoursmith => "Armoursmith",
            Self::PotionShop => "Potion shop",
            Self::TrainingHall => "Training hall",
            Self::ScrollShop => "Scroll shop",
            Self::AdventurersGuild => "Adventurers' guild",
            Self::Jeweller => "Jeweller",
            Self::Library => "Library",
            Self::Bank => "Bank",
            Self::GemCutter => "Gem cutter",
            Self::BlackMarket => "Black market",
            Self::Cathedral => "Cathedral",
        }
    }

    /// The fewest buildings a town needs to have one, and its weight in the
    /// smallest and largest towns.
    pub(super) fn odds(self) -> (i64, u64, u64) {
        match self {
            Self::House => (3, 0, 0), // Chosen first, not by weight.
            Self::Shrine => (3, 40, 5),
            Self::Inn => (3, 35, 8),
            Self::Tavern => (3, 25, 10),
            Self::Blacksmith => (3, 15, 10),
            Self::Herbalist => (3, 10, 8),
            Self::Carpenter => (6, 8, 8),
            Self::Tailor => (8, 6, 8),
            Self::Leatherworker => (8, 5, 8),
            Self::Fletcher => (10, 4, 7),
            Self::Church => (15, 2, 6),
            Self::Weaponsmith => (20, 1, 8),
            Self::Armoursmith => (20, 1, 8),
            Self::PotionShop => (12, 1, 8),
            Self::TrainingHall => (20, 1, 6),
            Self::ScrollShop => (25, 1, 6),
            Self::AdventurersGuild => (25, 1, 5),
            Self::Jeweller => (30, 1, 5),
            Self::Library => (30, 1, 4),
            Self::Bank => (30, 1, 4),
            Self::GemCutter => (40, 1, 4),
            Self::BlackMarket => (40, 1, 2),
            Self::Cathedral => (50, 1, 1),
        }
    }
}
