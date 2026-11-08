//! One of Health or Mana: how much there is, and refilling.

use serde::{Deserialize, Serialize};

use super::points::{PER_POINT, Points};

/// How much of one there is, and its practice. Refills in a straight line,
/// empty to full in `Limits::refill` (`REFILL_TICKS`), from the tick `since`. Kept in thousandths
/// (`Points`); the most is in whole points.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(from = "TrackSave", into = "TrackSave")]
pub struct Track {
    /// How much there was at `since`, in thousandths: `u32::MAX` is full,
    /// whatever the most.
    value: u32,
    /// The action tick it refills from: when it was last settled, or later
    /// while in combat.
    since: u64,
    /// Practice gained by using it, which raises its most, in thousandths.
    pub practice: u32,
    /// Practice gained in the current hour of playtime, in thousandths.
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

/// How a `Track` is saved and sent. Saves from before thousandths have no
/// `milli`, and their whole points are turned into thousandths on reading.
#[derive(Serialize, Deserialize)]
struct TrackSave {
    value: u32,
    since: u64,
    practice: u32,
    this_hour: u32,
    #[serde(default)]
    milli: bool,
}

impl From<TrackSave> for Track {
    fn from(save: TrackSave) -> Self {
        let scale = |value: u32| {
            if save.milli {
                value
            } else {
                value.saturating_mul(PER_POINT)
            }
        };
        Self {
            // Full stays full.
            value: if save.value == u32::MAX {
                u32::MAX
            } else {
                scale(save.value)
            },
            since: save.since,
            practice: scale(save.practice),
            this_hour: scale(save.this_hour),
        }
    }
}

impl From<Track> for TrackSave {
    fn from(track: Track) -> Self {
        Self {
            value: track.value,
            since: track.since,
            practice: track.practice,
            this_hour: track.this_hour,
            milli: true,
        }
    }
}

/// What a client is told of one at a moment: it refills it itself from there
/// with `refilled`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct VitalNow {
    pub value: Points,
    /// In whole points.
    pub most: u32,
    /// Action ticks from now until it starts refilling.
    pub refills_in: u32,
    /// Action ticks to refill from nothing to full (`Limits::refill`).
    pub refill: u32,
}

impl VitalNow {
    /// How much there is this many action ticks after it was told.
    pub fn after(self, ticks: u64) -> Points {
        refilled(
            self.value,
            self.most,
            self.refill.into(),
            ticks.saturating_sub(self.refills_in.into()),
        )
    }
}

/// `value` out of `most` (whole points) after refilling for this many action
/// ticks, at `refill` ticks from nothing to full.
pub fn refilled(value: Points, most: u32, refill: u64, ticks: u64) -> Points {
    let most = u64::from(most) * u64::from(PER_POINT);
    let value = u64::from(value.milli()).min(most);
    let gained = ticks.saturating_mul(most) / refill.max(1);
    Points(value.saturating_add(gained).min(most) as u32)
}

impl Track {
    /// How much there is at this tick, out of `most` (whole points), refilling
    /// from nothing to full in `refill` ticks.
    pub fn at(&self, most: u32, refill: u64, now: u64) -> Points {
        refilled(Points(self.value), most, refill, now.saturating_sub(self.since))
    }

    /// What a client is told of it at this tick.
    pub fn told(&self, most: u32, refill: u64, now: u64) -> VitalNow {
        VitalNow {
            value: self.at(most, refill, now),
            most,
            refills_in: self.since.saturating_sub(now).min(u32::MAX.into()) as u32,
            refill: refill.min(u32::MAX.into()) as u32,
        }
    }

    /// Fixes it at what it is now; a wait for refilling stays. Full stays
    /// full, whatever the most comes to be.
    pub(super) fn settle(&mut self, most: u32, refill: u64, now: u64) {
        let value = self.at(most, refill, now);
        self.value = if value >= Points::whole(most) {
            u32::MAX
        } else {
            value.milli()
        };
        self.since = self.since.max(now);
    }

    /// Takes this much away (settled first) out of `most`, never below 0.
    pub(super) fn take(&mut self, amount: Points, most: u32) {
        self.value = self
            .value
            .min(Points::whole(most).milli())
            .saturating_sub(amount.milli());
    }

    /// Adds this much (settled first) out of `most`: full at the most, and
    /// what is beyond it lost.
    pub(super) fn give(&mut self, amount: Points, most: u32) {
        let most = Points::whole(most).milli();
        let value = self.value.min(most).saturating_add(amount.milli());
        self.value = if value >= most { u32::MAX } else { value };
    }

    /// No refilling before this tick.
    pub(super) fn calm_from(&mut self, tick: u64) {
        self.since = self.since.max(tick);
    }
}
