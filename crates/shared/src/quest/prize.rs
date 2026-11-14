//! What the artifact line's end pays: an artifact, or the planet's own set
//! item (decisions.md: one per planet, from the planet's seed, so players may
//! later learn what each planet has). Only weapons, armour pieces and
//! jewellery: things with quality worth carrying for good (Claude's choice).

use crate::item::{ItemKind, Quality};
use crate::rng::Seed;

/// The kinds an artifact or a set item may be: weapons, armour pieces (not
/// whole suits, nor a horse's) and jewellery.
pub const PRIZE_KINDS: [ItemKind; 33] = [
    ItemKind::Axe,
    ItemKind::Dagger,
    ItemKind::Knife,
    ItemKind::Longsword,
    ItemKind::Polearm,
    ItemKind::ShortSword,
    ItemKind::Spear,
    ItemKind::Sword,
    ItemKind::ArmArmour,
    ItemKind::ArmingCap,
    ItemKind::ArmingVest,
    ItemKind::BevoredHelm,
    ItemKind::Chausses,
    ItemKind::FootArmour,
    ItemKind::Coif,
    ItemKind::Collar,
    ItemKind::Gorget,
    ItemKind::Bevor,
    ItemKind::Aventail,
    ItemKind::Mantle,
    ItemKind::DemiGauntlets,
    ItemKind::Gambeson,
    ItemKind::Gauntlets,
    ItemKind::Helmet,
    ItemKind::LegArmour,
    ItemKind::PaddedChausses,
    ItemKind::PaddedGloves,
    ItemKind::ShoulderArmour,
    ItemKind::TorsoArmour,
    ItemKind::CuirassierTassets,
    ItemKind::WaistArmour,
    ItemKind::Necklace,
    ItemKind::Ring,
];

/// How good a planet's set item is (decisions.md: any quality; Claude's
/// weights, never Poor or Artifact).
const SET_QUALITIES: [(Quality, u64); 5] = [
    (Quality::Common, 10),
    (Quality::Uncommon, 25),
    (Quality::Rare, 30),
    (Quality::Epic, 25),
    (Quality::Legendary, 10),
];

/// The planet's set item, from its seed (`WorldSeed::value`): its kind and
/// quality, the same for everyone.
pub fn set_item_of(planet: u64) -> (ItemKind, Quality) {
    let mut rng = Seed::new(planet).derive("set item").rng();
    let kind = PRIZE_KINDS[rng.range_u64(0, PRIZE_KINDS.len() as u64) as usize];
    (kind, rng.pick_weighted(&SET_QUALITIES))
}

/// An artifact's kind, from a seed of its own (the new item's).
pub fn artifact_kind(seed: Seed) -> ItemKind {
    let mut rng = seed.derive("artifact kind").rng();
    PRIZE_KINDS[rng.range_u64(0, PRIZE_KINDS.len() as u64) as usize]
}

#[cfg(test)]
mod tests;
