//! `Effects`: every effect on one creature (saved on `Sheet`).

use serde::{Deserialize, Serialize};

use super::effect::{Cancel, Effect};
use super::stat::Bonus;

/// The effects on one creature, at most one from each `Source`. Changing them
/// changes the most Health and Mana, so a `Sheet` does it (`Sheet::change_effects`),
/// settling its vitals first.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Effects(pub Vec<Effect>);

impl Effects {
    /// Lays one on, in place of any from the same source.
    pub fn add(&mut self, effect: Effect) {
        self.0.retain(|old| old.source != effect.source);
        self.0.push(effect);
    }

    /// Ends those that this ends. Whether any were.
    pub fn cancel(&mut self, cancel: Cancel) -> bool {
        let before = self.0.len();
        self.0.retain(|effect| !effect.ends_on.by(cancel));
        self.0.len() != before
    }

    /// Whether any would be ended by this.
    pub fn cancelled_by(&self, cancel: Cancel) -> bool {
        self.0.iter().any(|effect| effect.ends_on.by(cancel))
    }

    /// Forgets those that have ended by `now`.
    pub fn forget_ended(&mut self, now: u64) {
        self.0.retain(|effect| effect.lasts(now));
    }

    /// The earliest end tick at or before `now`, of one not yet forgotten.
    pub fn first_ended(&self, now: u64) -> Option<u64> {
        self.0
            .iter()
            .filter_map(|effect| effect.end)
            .filter(|&end| end <= now)
            .min()
    }

    /// The earliest end after `now` among those lasting, if any ends.
    pub fn next_end(&self, now: u64) -> Option<u64> {
        self.0
            .iter()
            .filter_map(|effect| effect.end)
            .filter(|&end| end > now)
            .min()
    }

    /// The bonuses of every effect lasting at `now`.
    pub fn bonuses(&self, now: u64) -> impl Iterator<Item = &Bonus> {
        self.0
            .iter()
            .filter(move |effect| effect.lasts(now))
            .flat_map(|effect| &effect.bonuses)
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}
