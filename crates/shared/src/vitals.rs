//! Health and Mana: how much harm a creature can take and how much magic it has
//! left. Each has a most, from an attribute, playtime, practice and changes
//! laid on it, and refills by itself once out of combat. The same rules for
//! players and, later, mobs. See `docs/context/shared/vitals.md`.
//!
//! Like `energy`, only a value and the tick it counts from are stored: the
//! formula gives any later tick, so nothing loops over creatures and time.
//! Numbers are first guesses, to balance by playing.

use serde::{Deserialize, Serialize};

use crate::energy::ACTION_TICKS_PER_SECOND;
use crate::sheet::{Attribute, Attributes};

mod limits;
mod points;
mod track;

pub use limits::{Limits, MostParts};
pub use points::{PER_POINT, Points};
pub use track::{Track, VitalNow, refilled};

/// Action ticks in a second, as a whole number.
const TICKS: u64 = ACTION_TICKS_PER_SECOND as u64;

/// Attribute points for each point of the most: a median 100 gives 20.
pub const ATTRIBUTE_PER_POINT: u32 = 5;
/// How long refilling from nothing to full takes: 5 minutes (owner, 2026-11-06).
pub const REFILL_TICKS: u64 = 5 * 60 * TICKS;
/// How long after the last fighting Health starts refilling: 10 seconds.
pub const CALM_AFTER: u64 = 10 * TICKS;
/// Playtime for each point added to both: 24 hours (owner, 2026-11-06).
pub const PLAYTIME_PER_POINT: u64 = 24 * 60 * 60 * TICKS;
/// Practice (whole points of it) is turned into points by a square root of it
/// in steps of this: 25 gives 1 point, 100 gives 2, 2500 gives 10.
pub const PRACTICE_STEP: u32 = 25;
/// The most practice each can gain in an hour of playtime, in whole points.
pub const PRACTICE_PER_HOUR: u32 = 20;
/// An hour of playtime, for that limit.
const PLAYTIME_HOUR: u64 = 60 * 60 * TICKS;

/// One of the two.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Vital {
    Health,
    Mana,
}

impl Vital {
    pub const ALL: [Self; 2] = [Self::Health, Self::Mana];

    pub const fn name(self) -> &'static str {
        match self {
            Self::Health => "Health",
            Self::Mana => "Mana",
        }
    }

    /// The attribute its most comes from.
    pub const fn attribute(self) -> Attribute {
        match self {
            Self::Health => Attribute::Toughness,
            Self::Mana => Attribute::Will,
        }
    }
}

/// Something raising or lowering the most of one, for a while or for good
/// (potions, curses, blessings; for now only dev tools make them).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Change {
    pub vital: Vital,
    pub amount: i32,
    /// The action tick it ends at, or `None` for good.
    pub until: Option<u64>,
}

impl Change {
    /// Whether it still counts at this tick.
    pub fn lasts(&self, now: u64) -> bool {
        self.until.is_none_or(|until| now < until)
    }
}

/// One creature's Health and Mana. A new one (and one missing from an older
/// save) is full.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct Vitals {
    pub health: Track,
    pub mana: Track,
    /// The hour of playtime practice was last gained in, for
    /// `PRACTICE_PER_HOUR`.
    practice_hour: u64,
    pub changes: Vec<Change>,
}

impl Vitals {
    pub fn track(&self, vital: Vital) -> &Track {
        match vital {
            Vital::Health => &self.health,
            Vital::Mana => &self.mana,
        }
    }

    fn track_mut(&mut self, vital: Vital) -> &mut Track {
        match vital {
            Vital::Health => &mut self.health,
            Vital::Mana => &mut self.mana,
        }
    }

    /// How much of it there is at this tick.
    pub fn at(&self, vital: Vital, limits: Limits, now: u64) -> Points {
        self.track(vital).at(limits.get(vital), now)
    }

    /// What the client is told of it at this tick.
    pub fn told(&self, vital: Vital, limits: Limits, now: u64) -> VitalNow {
        self.track(vital).told(limits.get(vital), now)
    }

    /// Fixes both at what they are now: before anything changes how much
    /// there can be.
    pub fn settle(&mut self, limits: Limits, now: u64) {
        for vital in Vital::ALL {
            self.track_mut(vital).settle(limits.get(vital), now);
        }
    }

    /// Fills both.
    pub fn fill(&mut self) {
        self.health = Track::default();
        self.mana = Track::default();
    }

    /// Takes this much harm: neither refills for a while, and the harm is
    /// practice for Health. Health never goes below 0.
    pub fn hurt(&mut self, amount: Points, limits: Limits, now: u64, playtime: u64) {
        self.in_combat(limits, now);
        self.health.take(amount, limits.health);
        self.practise(Vital::Health, amount, playtime);
    }

    /// Spends this much Mana, if there is that much (`false` if not): what is
    /// spent is practice for Mana. Mana keeps refilling, and casting isn't
    /// fighting (owner, 2026-11-07).
    pub fn spend_mana(&mut self, amount: Points, limits: Limits, now: u64, playtime: u64) -> bool {
        if self.at(Vital::Mana, limits, now) < amount {
            return false;
        }
        self.settle(limits, now);
        self.mana.take(amount, limits.mana);
        self.practise(Vital::Mana, amount, playtime);
        true
    }

    /// Gives back this much of one, never past the most; what is beyond it is
    /// lost. A wait for refilling stays.
    pub fn restore(&mut self, vital: Vital, amount: Points, limits: Limits, now: u64) {
        self.settle(limits, now);
        self.track_mut(vital).give(amount, limits.get(vital));
    }

    /// Fighting: Health doesn't refill until `CALM_AFTER` from now. Mana always
    /// refills (owner, 2026-11-07).
    pub fn in_combat(&mut self, limits: Limits, now: u64) {
        self.settle(limits, now);
        self.health.calm_from(now + CALM_AFTER);
    }

    /// Lays a change on, settling first.
    pub fn add_change(&mut self, change: Change, limits: Limits, now: u64) {
        self.settle(limits, now);
        self.changes.push(change);
    }

    /// Forgets the changes that have ended by `now`, settling first at what
    /// they were (`limits` with them). Whether any had.
    pub fn forget_ended(&mut self, limits: Limits, now: u64) -> bool {
        if self.changes.iter().all(|change| change.lasts(now)) {
            return false;
        }
        self.settle(limits, now);
        self.changes.retain(|change| change.lasts(now));
        true
    }

    /// Practice from use, as much of it as this hour of playtime has room for.
    fn practise(&mut self, vital: Vital, amount: Points, playtime: u64) {
        let hour = playtime / PLAYTIME_HOUR;
        if hour != self.practice_hour {
            self.practice_hour = hour;
            self.health.this_hour = 0;
            self.mana.this_hour = 0;
        }
        let track = self.track_mut(vital);
        let room = Points::whole(PRACTICE_PER_HOUR).milli();
        let gained = amount.milli().min(room.saturating_sub(track.this_hour));
        track.this_hour += gained;
        track.practice += gained;
    }

    /// The most of each, and where it comes from (`limits`).
    pub fn most(
        &self,
        vital: Vital,
        attributes: &Attributes,
        playtime: u64,
        now: u64,
    ) -> MostParts {
        MostParts {
            attribute: u32::from(attributes.get(vital.attribute())) / ATTRIBUTE_PER_POINT,
            playtime: (playtime / PLAYTIME_PER_POINT) as u32,
            practice: (self.track(vital).practice / Points::whole(PRACTICE_STEP).milli()).isqrt(),
            changes: self
                .changes
                .iter()
                .filter(|change| change.vital == vital && change.lasts(now))
                .map(|change| change.amount)
                .sum(),
        }
    }

    /// The most of both now.
    pub fn limits(&self, attributes: &Attributes, playtime: u64, now: u64) -> Limits {
        Limits {
            health: self.most(Vital::Health, attributes, playtime, now).total(),
            mana: self.most(Vital::Mana, attributes, playtime, now).total(),
        }
    }
}

#[cfg(test)]
mod tests;
