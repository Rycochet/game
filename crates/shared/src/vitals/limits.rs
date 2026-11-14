//! How much Health and Mana there can be, and how fast they come back.

use serde::{Deserialize, Serialize};

use super::Vital;
use super::track::Rate;

/// The most of each, worked out at one tick (`Vitals::limits`), and how fast
/// they refill.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Limits {
    pub health: u32,
    pub mana: u32,
    /// Action ticks to refill from nothing to full: `REFILL_TICKS`, or less
    /// on Easy (`Difficulty::refill_ticks`). 0 counts as 1.
    pub refill: u64,
    /// What bonuses give back on top of refilling (`bonus::Stat::Restore`).
    pub restore: Restore,
}

/// Health and Mana given back over time by bonuses, in thousandths of a point
/// a second, while refilling, until a tick (when the effects giving it are
/// worked out again) or for good.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Restore {
    pub health: u32,
    pub mana: u32,
    pub until: Option<u64>,
}

impl Limits {
    pub fn get(self, vital: Vital) -> u32 {
        match vital {
            Vital::Health => self.health,
            Vital::Mana => self.mana,
        }
    }

    /// How one comes back.
    pub fn rate(self, vital: Vital) -> Rate {
        Rate {
            most: self.get(vital),
            refill: self.refill,
            restore: match vital {
                Vital::Health => self.restore.health,
                Vital::Mana => self.restore.mana,
            },
            until: self.restore.until,
        }
    }
}

/// Where the most of one comes from, to show and to add up.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct MostParts {
    pub vital: Vital,
    /// From its attribute (`ATTRIBUTE_PER_POINT`), with the attribute's bonuses.
    pub attribute: u32,
    /// From playtime (`PLAYTIME_PER_POINT`).
    pub playtime: u32,
    /// From use (`PRACTICE_STEP`).
    pub practice: u32,
    /// From the most's own bonuses, which can lower it.
    pub bonus: i32,
}

impl MostParts {
    /// All of it: Health never below 1, Mana never below 0 (armour can take it
    /// all; owner, 2026-11-14).
    pub fn total(self) -> u32 {
        let least = match self.vital {
            Vital::Health => 1,
            Vital::Mana => 0,
        };
        let total = i64::from(self.attribute)
            + i64::from(self.playtime)
            + i64::from(self.practice)
            + i64::from(self.bonus);
        total.clamp(least, i64::from(u32::MAX)) as u32
    }
}
