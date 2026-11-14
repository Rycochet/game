//! Keeping Health's and Mana's share when bonuses change the most.

use super::{Limits, Vital, Vitals};

impl Vitals {
    /// Bonuses have changed the most from `before` to `after` (the same
    /// playtime and practice): settles at `before`, then keeps each one's
    /// share of its most, rounding down, so 50 of 100 is 60 of 120 and back to
    /// 50 of 100, and putting things on or off never heals or refills (owner,
    /// 2026-11-14). Full stays full. Growth from playtime and practice doesn't
    /// come through here: below full stays the same number.
    pub fn rebase(&mut self, before: Limits, after: Limits, now: u64) {
        self.settle(before, now);
        for vital in Vital::ALL {
            let (old, new) = (before.get(vital), after.get(vital));
            if old != new {
                self.track_mut(vital).rescale(old, new);
            }
        }
    }
}
