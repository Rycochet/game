//! What an attack costs in energy (`shared::energy`), and how much of it is the
//! wind-up before it lands. All factors are per mille (thousandths), so the sums
//! stay whole numbers and both sides agree.

use super::Weapon;

/// An attack with bare hands: two a second at `NORMAL_SPEED` (210 × 20 energy a
/// second), the fastest there is.
pub const UNARMED_COST: u32 = 2100;
/// Energy for each gram of handling weight, in thousandths: a 1.2 kg sword comes
/// to one attack a second, a polearm about one every two.
pub const COST_PER_GRAM_MILLI: u64 = 1750;
/// A median creature's attribute.
pub const MEDIAN: u64 = 100;
/// The most a skilled hand takes off an attack's cost, per mille: 15%.
pub const SKILL_SAVING: u64 = 150;
/// The most dual wielding skill takes off each hand's attack, per mille: 20%, so
/// attacks come up to 25% faster.
pub const DUAL_SAVING: u64 = 200;
/// The share of an attack's cost waited before it lands (the wind-up), per mille;
/// the rest is the recovery after it.
pub const ATTACK_WIND_UP: u64 = 250;

/// How much strength lightens a weapon, per mille: the median's is 1000, twice
/// as strong halves it, never past half or double.
pub fn strength_lightens(strength: u16) -> u64 {
    (MEDIAN * 1000 / u64::from(strength.max(1))).clamp(500, 2000)
}

/// What the weapon alone costs to swing, before strength and skill: bare hands
/// plus its handling weight. Also how hard it hits (`damage::weapon_base`).
pub fn weapon_cost(weapon: &Weapon) -> u64 {
    u64::from(UNARMED_COST) + u64::from(weapon.handling_grams()) * COST_PER_GRAM_MILLI / 1000
}

/// A skill as a percentage for the rules: its value along its own node, in whole
/// points, at most 100 (specialising may later go past it).
pub fn skill_percent(value_hundredths: u32) -> u64 {
    (u64::from(value_hundredths) / 100).min(100)
}

/// What one attack costs: bare hands' cost, plus the weapon's handling weight
/// lightened by strength, then up to `SKILL_SAVING` off by skill with the weapon
/// (a percentage, `skill_percent`). With a weapon in each hand, `dual` is the
/// dual wielding skill's percentage, taking up to `DUAL_SAVING` more off.
pub fn attack_cost(weapon: &Weapon, strength: u16, skill: u64, dual: Option<u64>) -> u32 {
    let weight = weapon_cost(weapon) - u64::from(UNARMED_COST);
    let cost = u64::from(UNARMED_COST) + weight * strength_lightens(strength) / 1000;
    let cost = cost * (1000 - SKILL_SAVING * skill.min(100) / 100) / 1000;
    let cost = match dual {
        Some(dual) => cost * (1000 - DUAL_SAVING * dual.min(100) / 100) / 1000,
        None => cost,
    };
    cost.max(1) as u32
}

/// How much of a cost is the wind-up, waited before the attack lands.
pub fn wind_up(cost: u32) -> u32 {
    (u64::from(cost) * ATTACK_WIND_UP / 1000) as u32
}
