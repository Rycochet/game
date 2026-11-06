//! What dev tools can do to a character's Health and Mana.

use serde::{Deserialize, Serialize};

/// What `ClientMessage::DevVitals` does to the character's Health and Mana.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum DevVitals {
    /// Takes this much Health, as harm would: practice, and no refilling for a while.
    Hurt(u32),
    /// Spends this much Mana, if there is that much.
    SpendMana(u32),
    /// Fills both.
    Fill,
    /// Raises the most of both by `DEV_BONUS` for `DEV_BONUS_TICKS`.
    Bonus,
}

/// What `DevVitals::Bonus` adds to the most of each.
pub const DEV_BONUS: i32 = 5;
/// How long `DevVitals::Bonus` lasts: 5 minutes of action ticks.
pub const DEV_BONUS_TICKS: u64 = 5 * 60 * crate::energy::ACTION_TICKS_PER_SECOND as u64;
