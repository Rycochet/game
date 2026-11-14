//! `Effect`: bonuses laid on a creature for a while or for good, by eating,
//! drinking or dev tools, and what ends them.

use serde::{Deserialize, Serialize};

use super::stat::{Bonus, Stat};
use crate::item::ItemKind;
use crate::vitals::Vital;

/// Where an effect came from. A creature has at most one from each: a new one
/// takes its place, so eating a second apple starts it again.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Source {
    /// Eating or drinking one of these.
    Eaten(ItemKind),
    /// The debug menu's Bonus (`protocol::DevVitals::Bonus`), and the changes
    /// older saves kept on `Vitals`.
    Dev,
}

/// Something that ends an effect early (`EndsOn`).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Cancel {
    /// Taking a step or moving at all.
    Moving,
    /// Being harmed.
    Hurt,
}

/// What ends an effect before its end tick.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct EndsOn {
    pub moving: bool,
    pub hurt: bool,
}

impl EndsOn {
    pub const NOTHING: Self = Self {
        moving: false,
        hurt: false,
    };

    /// Whether this ends it.
    pub const fn by(self, cancel: Cancel) -> bool {
        match cancel {
            Cancel::Moving => self.moving,
            Cancel::Hurt => self.hurt,
        }
    }
}

/// Bonuses that last from `start` until `end` (an action tick, so they wear
/// off in real time, offline too), or for good with no end, unless something
/// in `ends_on` happens first.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Effect {
    pub source: Source,
    pub bonuses: Vec<Bonus>,
    /// The action tick it began at.
    pub start: u64,
    /// The action tick it ends at, or `None` for good.
    pub end: Option<u64>,
    pub ends_on: EndsOn,
}

impl Effect {
    /// Whether it still counts at this tick.
    pub fn lasts(&self, now: u64) -> bool {
        self.end.is_none_or(|end| now < end)
    }

    /// The debug menu's Bonus: `amount` more of the most of both, from `now`
    /// until `end`.
    pub fn dev_bonus(amount: i32, now: u64, end: u64) -> Self {
        Self {
            source: Source::Dev,
            bonuses: Vital::ALL
                .into_iter()
                .map(|vital| Bonus::flat(Stat::Most(vital), amount.saturating_mul(1000)))
                .collect(),
            start: now,
            end: Some(end),
            ends_on: EndsOn::NOTHING,
        }
    }
}
