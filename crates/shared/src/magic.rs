//! Magic: spells, the abilities that spend Mana. Heal so far. See
//! `docs/context/shared/magic.md`.

mod heal;

pub use heal::{
    AUTO_HEAL_BELOW, HEAL_COST, HEAL_MANA, HEAL_RANGE, HEAL_SKILL, HEAL_WIND_UP, PRACTICE_PER_CAST,
    RATIO_AT_BEST, RATIO_AT_FIRST, heal_amount, heal_ratio, heal_reaches, heal_skill,
    needs_healing,
};

#[cfg(test)]
mod tests;
