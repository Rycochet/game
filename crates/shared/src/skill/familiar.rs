//! Familiar foes teach less (owner, 2026-11-07): the more 5-second blocks spent
//! fighting one kind of creature (`sheet::Foes`), the less practice a blow against
//! it gives a skill that hasn't kept up, so farming one kind stalls.

use super::VALUE_SCALE;

/// Every this much familiarity above the skill's own value halves practice: 10%.
pub const FAMILIAR_HALVING: u32 = 10 * VALUE_SCALE;

/// How familiar a kind fought for this many blocks is, in hundredths like skill
/// values: the square root of the blocks, so 100 blocks is 10% and 10,000 is 100%.
/// It grows faster than a skill's cube root, so it catches up with farming.
pub fn familiarity(fought: u32) -> u32 {
    let scale = u64::from(VALUE_SCALE);
    (u64::from(fought) * scale * scale).isqrt() as u32
}

/// How much of `amount` a blow teaches a skill whose own value is `value`, against
/// a foe this familiar: all of it while familiarity is no higher than the value,
/// then half for every `FAMILIAR_HALVING` above it, falling in a straight line
/// between halvings, rounded to the nearest whole.
pub fn taught(amount: u32, familiarity: u32, value: u32) -> u32 {
    let over = familiarity.saturating_sub(value);
    let halvings = over / FAMILIAR_HALVING;
    if halvings >= 32 {
        return 0;
    }
    let rest = u64::from(over % FAMILIAR_HALVING);
    let whole = 2 * u64::from(FAMILIAR_HALVING);
    let below = whole << halvings;
    ((u64::from(amount) * (whole - rest) + below / 2) / below) as u32
}

#[cfg(test)]
mod tests;
