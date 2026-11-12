//! A creature's hidden alignment: how good or evil it has shown itself to be,
//! from −1000 (evil) to +1000 (good), moved only by its deeds (owner,
//! 2026-11-12: no drift over time). Never sent to any client: only developer
//! tools show it. See `docs/context/shared/alignment.md`.
//!
//! Players always have one, and so does every people kind; animals have none,
//! as they act on instinct rather than by choice (owner), until they are
//! someone's pet.

use serde::{Deserialize, Serialize};

mod deed;
mod kinds;

pub use deed::Deed;

/// The most evil an alignment can be.
pub const MOST_EVIL: i16 = -1000;
/// The most good an alignment can be.
pub const MOST_GOOD: i16 = 1000;
/// At or below this an alignment is Evil. *To balance.*
pub const EVIL_FROM: i16 = -300;
/// At or above this an alignment is Good. *To balance.*
pub const GOOD_FROM: i16 = 300;

/// How good or evil a creature is: 0 to begin with, always within `MOST_EVIL`
/// and `MOST_GOOD`.
#[derive(
    Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize,
)]
pub struct Alignment(i16);

/// Which side of the line an alignment is on. What quests and later effects go
/// by, never the number itself.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum Band {
    Evil,
    Neutral,
    Good,
}

impl Band {
    pub const fn name(self) -> &'static str {
        match self {
            Self::Evil => "Evil",
            Self::Neutral => "Neutral",
            Self::Good => "Good",
        }
    }
}

impl Alignment {
    /// Neither good nor evil: where everyone starts.
    pub const NEUTRAL: Self = Self(0);

    /// This value, kept within `MOST_EVIL` and `MOST_GOOD`.
    pub fn new(value: i32) -> Self {
        Self(value.clamp(i32::from(MOST_EVIL), i32::from(MOST_GOOD)) as i16)
    }

    pub const fn value(self) -> i16 {
        self.0
    }

    pub const fn band(self) -> Band {
        if self.0 <= EVIL_FROM {
            Band::Evil
        } else if self.0 >= GOOD_FROM {
            Band::Good
        } else {
            Band::Neutral
        }
    }

    /// Moved by this much, kept within its limits.
    pub fn shifted(self, by: i32) -> Self {
        Self::new(i32::from(self.0) + by)
    }

    /// Moved by what this deed is worth (`Deed::shift`).
    pub fn after(self, deed: Deed) -> Self {
        self.shifted(deed.shift())
    }
}

#[cfg(test)]
mod tests;
