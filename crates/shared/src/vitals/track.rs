//! One of Health or Mana: how much there is, and refilling.

use serde::{Deserialize, Serialize};

use super::REFILL_TICKS;

/// How much of one there is, and its practice. Refills in a straight line,
/// empty to full in `REFILL_TICKS`, from the tick `since`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Track {
    /// How much there was at `since`: `u32::MAX` is full, whatever the most.
    value: u32,
    /// The action tick it refills from: when it was last settled, or later
    /// while in combat.
    since: u64,
    /// Practice gained by using it, which raises its most.
    pub practice: u32,
    /// Practice gained in the current hour of playtime.
    pub(super) this_hour: u32,
}

impl Default for Track {
    fn default() -> Self {
        Self {
            value: u32::MAX,
            since: 0,
            practice: 0,
            this_hour: 0,
        }
    }
}

/// What a client is told of one at a moment: it refills it itself from there
/// with `refilled`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct VitalNow {
    pub value: u32,
    pub most: u32,
    /// Action ticks from now until it starts refilling.
    pub refills_in: u32,
}

impl VitalNow {
    /// How much there is this many action ticks after it was told.
    pub fn after(self, ticks: u64) -> u32 {
        refilled(
            self.value,
            self.most,
            ticks.saturating_sub(self.refills_in.into()),
        )
    }
}

/// `value` out of `most` after refilling for this many action ticks.
pub fn refilled(value: u32, most: u32, ticks: u64) -> u32 {
    let value = value.min(most);
    let gained = ticks.saturating_mul(u64::from(most)) / REFILL_TICKS;
    u64::from(value).saturating_add(gained).min(u64::from(most)) as u32
}

impl Track {
    /// How much there is at this tick, out of `most`.
    pub fn at(&self, most: u32, now: u64) -> u32 {
        refilled(self.value, most, now.saturating_sub(self.since))
    }

    /// What a client is told of it at this tick.
    pub fn told(&self, most: u32, now: u64) -> VitalNow {
        VitalNow {
            value: self.at(most, now),
            most,
            refills_in: self.since.saturating_sub(now).min(u32::MAX.into()) as u32,
        }
    }

    /// Fixes it at what it is now; a wait for refilling stays. Full stays
    /// full, whatever the most comes to be.
    pub(super) fn settle(&mut self, most: u32, now: u64) {
        let value = self.at(most, now);
        self.value = if value >= most { u32::MAX } else { value };
        self.since = self.since.max(now);
    }

    /// Takes this much away (settled first) out of `most`, never below 0.
    pub(super) fn take(&mut self, amount: u32, most: u32) {
        self.value = self.value.min(most).saturating_sub(amount);
    }

    /// No refilling before this tick.
    pub(super) fn calm_from(&mut self, tick: u64) {
        self.since = self.since.max(tick);
    }
}
