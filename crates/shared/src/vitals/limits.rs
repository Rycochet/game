//! How much Health and Mana there can be.

use serde::{Deserialize, Serialize};

use super::Vital;

/// The most of each, worked out at one tick (`Vitals::limits`).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Limits {
    pub health: u32,
    pub mana: u32,
}

impl Limits {
    pub fn get(self, vital: Vital) -> u32 {
        match vital {
            Vital::Health => self.health,
            Vital::Mana => self.mana,
        }
    }
}

/// Where the most of one comes from, to show and to add up.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct MostParts {
    /// From its attribute (`ATTRIBUTE_PER_POINT`).
    pub attribute: u32,
    /// From playtime (`PLAYTIME_PER_POINT`).
    pub playtime: u32,
    /// From use (`PRACTICE_STEP`).
    pub practice: u32,
    /// From changes laid on it, which can lower it.
    pub changes: i32,
}

impl MostParts {
    /// All of it, never below 1.
    pub fn total(self) -> u32 {
        let total = i64::from(self.attribute)
            + i64::from(self.playtime)
            + i64::from(self.practice)
            + i64::from(self.changes);
        total.clamp(1, i64::from(u32::MAX)) as u32
    }
}
