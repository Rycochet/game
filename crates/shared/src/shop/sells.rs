//! What each kind of building sells, and how long each quality takes to make.

use super::MINUTES_PER_DAY;
use crate::item::{ItemKind, Quality};
use crate::site::BuildingKind;

/// What a kind of building sells, with how much each weighs: nothing if it isn't
/// a shop. Only placeholders exist to sell for now.
pub fn sells(kind: BuildingKind) -> &'static [(ItemKind, u64)] {
    use ItemKind::{
        Apple, ArmingCap, ArmingVest, Backpack, Belt, Boomerang, Boots, Braies, BreastBand, Cloak,
        Clogs, FingerlessGloves, Gauntlets, Gloves, Hat, Hose, Lantern, LinenCoif, Loincloth,
        Necklace, PaddedChausses, PaddedGloves, Petticoat, Pouch, Ring, Sandals, Shirt, Shoes,
        Slippers, SmallBox, Socks, Surcoat, Sword, Tabard, Trousers, Wimple,
    };
    match kind {
        BuildingKind::Blacksmith => &[(Sword, 2), (Gauntlets, 2), (Lantern, 1)],
        BuildingKind::Herbalist => &[(Apple, 3), (Pouch, 1)],
        BuildingKind::Carpenter => &[(SmallBox, 2), (Boomerang, 1), (Clogs, 1)],
        BuildingKind::Tailor => &[
            (Shirt, 3),
            (Braies, 2),
            (Hose, 2),
            (Trousers, 3),
            (Socks, 2),
            (Petticoat, 1),
            (BreastBand, 1),
            (Loincloth, 1),
            (LinenCoif, 1),
            (Wimple, 1),
            (Hat, 2),
            (Cloak, 1),
            (Gloves, 1),
            (FingerlessGloves, 1),
            (PaddedGloves, 1),
            (ArmingCap, 1),
            (ArmingVest, 1),
            (PaddedChausses, 1),
            (Surcoat, 1),
            (Tabard, 1),
            (Slippers, 1),
        ],
        BuildingKind::Leatherworker => &[
            (Belt, 2),
            (Boots, 2),
            (Shoes, 2),
            (Sandals, 1),
            (Pouch, 2),
            (Backpack, 1),
            (Gloves, 1),
        ],
        BuildingKind::Fletcher => &[(Boomerang, 1)],
        BuildingKind::Weaponsmith => &[(Sword, 2), (Boomerang, 1)],
        BuildingKind::Armoursmith => super::armoursmith::ARMOURSMITH,
        BuildingKind::PotionShop => &[(Apple, 1), (SmallBox, 1)],
        BuildingKind::ScrollShop => &[(Lantern, 1), (SmallBox, 1)],
        BuildingKind::Jeweller => &[(Ring, 2), (Necklace, 1)],
        BuildingKind::GemCutter => &[(Ring, 1), (Necklace, 1)],
        BuildingKind::BlackMarket => &[(Sword, 1), (Ring, 1), (Necklace, 1), (Cloak, 1)],
        _ => &[],
    }
}

/// How many steps better than Common this is, or worse.
pub(super) const fn steps_from_common(quality: Quality) -> i64 {
    quality as i64 - Quality::Common as i64
}

/// How long something of this quality takes to make, in game minutes, before
/// anything else. Nothing here makes artifacts: they take a questline.
pub(super) const fn base_minutes(quality: Quality) -> u64 {
    match quality {
        Quality::Poor => 30,
        Quality::Common => 60,
        Quality::Uncommon => 4 * 60,
        Quality::Rare => MINUTES_PER_DAY,
        Quality::Epic => 7 * MINUTES_PER_DAY,
        Quality::Legendary | Quality::Artifact => 30 * MINUTES_PER_DAY,
    }
}

/// How likely something in stock is to be the one sold, as a weight: the better,
/// the longer it stays, but a legendary still goes half as readily as a common.
pub(super) const fn sell_weight(quality: Quality) -> u64 {
    match quality {
        Quality::Poor => 110,
        Quality::Common => 100,
        Quality::Uncommon => 88,
        Quality::Rare => 75,
        Quality::Epic => 62,
        Quality::Legendary | Quality::Artifact => 50,
    }
}
