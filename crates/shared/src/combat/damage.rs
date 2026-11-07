//! How much harm an attack that lands does, in three stages: the blow itself, the
//! attacker's power against the defender's Defence, then armour. Factors are per
//! mille; the result is `Points` (thousandths of Health). Numbers are first
//! guesses, to balance by playing (owner, 2026-11-07).

use crate::item::Quality;
use crate::rng::Rng;
use crate::vitals::Points;

use super::Weapon;
use super::cost::{MEDIAN, UNARMED_COST, weapon_cost};

/// What a fist does before anything else counts: one Health.
pub const UNARMED_DAMAGE: u64 = 1000;
/// Weapons cut or stab: this much more than a fist, per mille.
pub const EDGE: u64 = 1200;
/// Armour's half point: this much armour takes half of what gets through.
pub const ARMOUR_HALF: u64 = 100;

/// Quality counts most (owner): Poor to Artifact, per mille.
pub const fn quality_factor(quality: Quality) -> u64 {
    match quality {
        Quality::Poor => 800,
        Quality::Common => 1000,
        Quality::Uncommon => 1150,
        Quality::Rare => 1300,
        Quality::Epic => 1500,
        Quality::Legendary => 1750,
        Quality::Artifact => 2000,
    }
}

/// Then the metal (`Material::tier`, 1 to 4): adamantine beats copper. Per mille.
pub const fn metal_factor(tier: u64) -> u64 {
    match tier {
        0 | 1 => 850,
        2 => 1000,
        3 => 1100,
        _ => 1250,
    }
}

/// How hard the weapon itself hits, per mille of a fist: as much more as it costs
/// to swing, so weight sets the rhythm (big slow blows or small quick ones, about
/// the same harm a second), while quality and metal raise it outright.
pub fn weapon_base(weapon: &Weapon) -> u64 {
    weapon_cost(weapon) * 1000 / u64::from(UNARMED_COST)
}

/// How strength adds to a blow, per mille: the square root of strength against
/// the median, so four times as strong hits twice as hard.
pub fn strength_hits(strength: u16) -> u64 {
    (u64::from(strength) * 1_000_000 / MEDIAN).isqrt()
}

/// The luck of one blow, per mille: two rolls added, from 500 to 1500, most often
/// near 1000, so wild blows are rare.
pub fn roll_blow(rng: &mut Rng) -> u64 {
    500 + rng.range_u64(0, 501) + rng.range_u64(0, 501)
}

/// Stage 1: the blow, in thousandths of Health, with its luck (`roll_blow`).
pub fn blow(weapon: &Weapon, strength: u16, luck: u64) -> u64 {
    let mut harm = UNARMED_DAMAGE * weapon_base(weapon) / 1000;
    harm = harm * strength_hits(strength) / 1000;
    if !weapon.is_unarmed() {
        harm = harm * EDGE / 1000;
        harm = harm * quality_factor(weapon.quality) / 1000;
        harm = harm * metal_factor(weapon.metal_tier) / 1000;
    }
    harm * luck / 1000
}

/// The off hand's share of a blow, per mille: half, and half the dual wielding
/// skill's percentage more, so a master's off hand hits as hard as the main.
pub fn off_hand_share(dual: u64) -> u64 {
    500 + dual.min(100) * 5
}

/// Stage 2: what gets past the defender's Defence, `attack ÷ (attack + defence)`
/// of it: equals take half, and the weaker side always lands something.
pub fn past_defence(harm: u64, attack: u64, defence: u64) -> u64 {
    harm * attack.max(1) / (attack.max(1) + defence)
}

/// Stage 3: what gets through armour, `ARMOUR_HALF ÷ (ARMOUR_HALF + armour)` of
/// it: never nothing, and each point of armour adds as much as the one before.
pub fn through_armour(harm: u64, armour: u64) -> u64 {
    harm * ARMOUR_HALF / (ARMOUR_HALF + armour)
}

/// All of an attack that lands, as Health lost.
pub struct Blow {
    pub weapon: Weapon,
    pub strength: u16,
    /// From `roll_blow`.
    pub luck: u64,
    /// The off hand's share (`off_hand_share`), or 1000 for the main hand.
    pub hand: u64,
    /// The attacker's power with the weapon (`power`).
    pub attack: u64,
    /// The defender's Defence power (`power`).
    pub defence: u64,
    pub armour: u64,
}

impl Blow {
    pub fn harm(&self) -> Points {
        let harm = blow(&self.weapon, self.strength, self.luck) * self.hand / 1000;
        let harm = past_defence(harm, self.attack, self.defence);
        let harm = through_armour(harm, self.armour);
        Points(harm.min(u64::from(u32::MAX)) as u32)
    }
}
