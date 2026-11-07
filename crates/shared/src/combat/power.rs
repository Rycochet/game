//! Power: how good a creature is at something, for setting one side against the
//! other. The plan's rule: attribute factor × skill value, so doubling either
//! doubles power (`docs/context/abilities-skills.md`, Checks).

use crate::sheet::{Attribute, Attributes};

/// The least a skill counts for, in hundredths, so someone untrained still has a
/// chance: a first guess.
pub const SKILL_FLOOR: u64 = 1000;

/// Power from the attributes a skill leans on (their average) and the skill's
/// total along its path (`Skills::total`, in hundredths).
pub fn power(attributes: &Attributes, leans: &[Attribute], total: u32) -> u64 {
    let leaning = if leans.is_empty() {
        100
    } else {
        let sum: u64 = leans.iter().map(|&a| u64::from(attributes.get(a))).sum();
        sum / leans.len() as u64
    };
    leaning.max(1) * u64::from(total).max(SKILL_FLOOR)
}
