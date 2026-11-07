//! Combat's rules: what an attack costs and how much harm it does, the same for
//! players and mobs. See `docs/context/shared/combat.md`, and the design in
//! `docs/context/abilities-skills.md`.

mod cost;
mod damage;
mod fighter;
mod power;
mod reach;
mod weapon;
mod wielded;

pub use cost::{
    ATTACK_WIND_UP, CANCEL_PENALTY, COST_PER_GRAM_MILLI, DUAL_SAVING, MEDIAN, SKILL_SAVING,
    UNARMED_COST, attack_cost, skill_percent, strength_lightens, weapon_cost, wind_up,
};
pub use damage::{
    ARMOUR_HALF, Blow, EDGE, UNARMED_DAMAGE, blow, metal_factor, off_hand_share, past_defence,
    quality_factor, roll_blow, strength_hits, through_armour, weapon_base,
};
pub use fighter::{Fighter, PRACTICE_PER_BLOW, Swing};
pub use power::{SKILL_FLOOR, power};
pub use reach::in_reach;
pub use weapon::Weapon;
pub use wielded::Wielded;

#[cfg(test)]
mod tests;
#[cfg(test)]
mod wielded_tests;
