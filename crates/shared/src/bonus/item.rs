//! What an item gives while worn: a pure function of the item (its kind,
//! quality, own weight and material), so both sides always agree.

use super::material::{MaterialFactors, armour_material, material_factors};
use super::stat::{Bonus, Stat};
use crate::item::{Item, ItemCategory, Quality};
use crate::sheet::Attribute;
use crate::vitals::Vital;

/// Toughness each kilogram of armour gives, at Common quality (owner, 2026-11-14).
pub const TOUGHNESS_PER_KG: i64 = 10;
/// Agility each kilogram of armour costs (owner, 2026-11-14).
pub const AGILITY_PER_KG: i64 = 2;
/// The most Mana each kilogram of armour costs (owner, 2026-11-14).
pub const MANA_PER_KG: i64 = 1;

/// How much better armour's Toughness is for its quality, in thousandths: Poor
/// 0.75, Common 1, a quarter more a step to Legendary 2, Artifact 2.5 (first
/// guesses). No quality counts as Common.
pub const fn quality_factor(quality: Option<Quality>) -> i64 {
    match quality {
        Some(Quality::Poor) => 750,
        None | Some(Quality::Common) => 1000,
        Some(Quality::Uncommon) => 1250,
        Some(Quality::Rare) => 1500,
        Some(Quality::Epic) => 1750,
        Some(Quality::Legendary) => 2000,
        Some(Quality::Artifact) => 2500,
    }
}

/// What this item gives while worn. Only armour gives anything so far: by
/// the kilogram of the piece's own weight (without anything in it), Toughness
/// by its quality and material, less Agility and most Mana by its material.
pub fn item_bonuses(item: &Item) -> Vec<Bonus> {
    if item.kind.category() != ItemCategory::Armour {
        return Vec::new();
    }
    let grams = i64::from(piece_grams(item));
    let material = armour_material(item).map_or(MaterialFactors::USUAL, material_factors);
    // Thousandths of a point per kilogram is one per gram.
    let toughness =
        TOUGHNESS_PER_KG * grams * quality_factor(item.quality) / 1000 * material.toughness / 1000;
    let agility = AGILITY_PER_KG * grams * material.agility / 1000;
    let mana = MANA_PER_KG * grams * material.mana / 1000;
    vec![
        Bonus::flat(Stat::Attribute(Attribute::Toughness), clamp(toughness)),
        Bonus::flat(Stat::Attribute(Attribute::Agility), clamp(-agility)),
        Bonus::flat(Stat::Most(Vital::Mana), clamp(-mana)),
    ]
}

/// One piece's own weight in grams: its form's, or its kind's.
fn piece_grams(item: &Item) -> u32 {
    item.form
        .as_ref()
        .and_then(|form| form.grams_at(item.made.size))
        .unwrap_or(item.kind.def().weight)
}

fn clamp(milli: i64) -> i32 {
    milli.clamp(i64::from(i32::MIN), i64::from(i32::MAX)) as i32
}
