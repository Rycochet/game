//! Health and Mana: how much harm a creature can take and how much magic it has
//! left. Each has a most, from an attribute, playtime, practice and bonuses
//! (`shared::bonus`), and refills by itself once out of combat. The same rules for
//! players and, later, mobs. See `docs/context/shared/vitals.md`.
//!
//! Like `energy`, only a value and the tick it counts from are stored: the
//! formula gives any later tick, so nothing loops over creatures and time.
//! Numbers are first guesses, to balance by playing.

use serde::{Deserialize, Serialize};

use crate::bonus::{Stat, Totals};
use crate::energy::ACTION_TICKS_PER_SECOND;
use crate::sheet::{Attribute, Attributes};

mod limits;
mod old_change;
mod points;
mod rebase;
mod track;

pub use limits::{Limits, MostParts, Restore};
pub(crate) use old_change::Change;
pub use points::{PER_POINT, Points};
pub use track::{Rate, Track, VitalNow, refilled};

/// Action ticks in a second, as a whole number.
pub(crate) const TICKS: u64 = ACTION_TICKS_PER_SECOND as u64;

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
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
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
    /// Older saves' changes to the most, which `Sheet::settled` turns into
    /// effects (`bonus::Effect`): always empty after that.
    pub(crate) changes: Vec<Change>,
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
        self.track(vital).at(limits.rate(vital), now)
    }

    /// What the client is told of it at this tick.
    pub fn told(&self, vital: Vital, limits: Limits, now: u64) -> VitalNow {
        self.track(vital).told(limits.rate(vital), now)
    }

    /// Fixes both at what they are now: before anything changes how much
    /// there can be.
    pub fn settle(&mut self, limits: Limits, now: u64) {
        for vital in Vital::ALL {
            self.track_mut(vital).settle(limits.rate(vital), now);
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

    /// Whether it is fighting at this tick: within `CALM_AFTER` of harming or
    /// being harmed, while Health waits to refill. No changing floors then
    /// (owner, 2026-11-08).
    pub fn fighting(&self, now: u64) -> bool {
        self.health.waiting(now)
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

    /// The most of each, and where it comes from (`limits`): `attributes` with
    /// their bonuses, and `totals` for the most's own.
    pub fn most(
        &self,
        vital: Vital,
        attributes: &Attributes,
        totals: &Totals,
        playtime: u64,
    ) -> MostParts {
        let attribute = u32::from(attributes.get(vital.attribute())) / ATTRIBUTE_PER_POINT;
        let playtime = (playtime / PLAYTIME_PER_POINT) as u32;
        let practice = (self.track(vital).practice / Points::whole(PRACTICE_STEP).milli()).isqrt();
        let base = i64::from(attribute) + i64::from(playtime) + i64::from(practice);
        let bonus = totals.whole(Stat::Most(vital), base) - base;
        MostParts {
            vital,
            attribute,
            playtime,
            practice,
            bonus: bonus.clamp(i64::from(i32::MIN), i64::from(i32::MAX)) as i32,
        }
    }

    /// The most of both with no bonuses, refilling at the usual rate: a mob's.
    pub fn limits(&self, attributes: &Attributes, playtime: u64) -> Limits {
        self.limits_with(attributes, &Totals::default(), playtime, None)
    }

    /// The most of both with these bonuses (`attributes` already with theirs),
    /// refilling at the usual rate, restoring as `totals` say until `until`
    /// (the next tick an effect ends, when it is worked out again).
    pub fn limits_with(
        &self,
        attributes: &Attributes,
        totals: &Totals,
        playtime: u64,
        until: Option<u64>,
    ) -> Limits {
        Limits {
            health: self
                .most(Vital::Health, attributes, totals, playtime)
                .total(),
            mana: self.most(Vital::Mana, attributes, totals, playtime).total(),
            refill: REFILL_TICKS,
            restore: Restore {
                health: totals.restoring(Vital::Health),
                mana: totals.restoring(Vital::Mana),
                until,
            },
        }
    }
}

#[cfg(test)]
mod restore_tests;
#[cfg(test)]
mod tests;
