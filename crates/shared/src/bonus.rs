//! Bonuses: what worn items and effects (food, potions, dev tools) add to a
//! creature's attributes, the most Health and Mana, skills, and things only
//! bonuses give (restoring Health or Mana over time). See
//! `docs/context/shared/bonus.md`.
//!
//! Totals are always added up again from everything that gives one, then
//! rounded once, so nothing builds up from adding and taking away a piece at a
//! time. Integers only: amounts are thousandths.

mod effect;
mod effects;
mod food;
mod item;
mod material;
mod stat;
mod totals;

pub use effect::{Cancel, Effect, EndsOn, Source};
pub use effects::Effects;
pub use food::{APPLE_RESTORE, APPLE_TICKS, food_effect, is_food};
pub use item::{
    AGILITY_PER_KG, HAIR_SHIRT_WILL, MANA_PER_KG, TOUGHNESS_PER_KG, item_bonuses, quality_factor,
};
pub use material::{MaterialFactors, material_factors};
pub use stat::{Amount, Bonus, PER_CENT, Stat};
pub use totals::{Sum, Totals, round_milli};

/// Thousandths in one whole unit of anything a bonus gives.
pub const MILLI: i64 = 1000;

#[cfg(test)]
mod tests;
