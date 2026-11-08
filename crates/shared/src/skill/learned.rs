//! `Learned`: what one creature keeps about one node: its practice, the most it has
//! ever had, and the playtimes rust goes by (`rust.rs`).

use serde::{Deserialize, Serialize};

use super::rust::Rust;

/// `used` before a server has seen the node: read from a save made before rust,
/// it is set to the character's playtime when they join (`Skills::settle`), so
/// nothing rusts at once for hours played before rust existed.
pub(super) const NOT_SEEN: u64 = u64::MAX;

/// One node a creature knows. Playtimes are in action ticks.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub(super) struct Learned {
    pub practice: u32,
    /// The most practice it has ever had: rust stops at a share of it, and
    /// practice below it is relearned twice as fast.
    pub best: u32,
    /// The playtime it was last used.
    pub used: u64,
    /// The playtime before which it can't rust again.
    pub clock: u64,
}

impl Learned {
    /// Adds `amount`, doubled while below the best it reached (relearning is
    /// faster than learning was), and raises the best.
    pub fn gain(&mut self, amount: u32) {
        let room = self.best.saturating_sub(self.practice);
        let fast = amount.min(room.div_ceil(2));
        let gained = fast
            .saturating_mul(2)
            .min(room)
            .saturating_add(amount - fast);
        self.practice = self.practice.saturating_add(gained);
        self.best = self.best.max(self.practice);
    }

    /// Where rust stops: a share of the best, never below 1 so a known node stays
    /// known.
    pub fn floor(&self, rust: Rust) -> u32 {
        let share = u64::from(self.best) * u64::from(rust.floor_per_mille) / 1000;
        (share as u32).max(1)
    }

    /// Takes `steps` of rust: each a share of what is left above the floor, at
    /// least 1 while there is any.
    pub fn rust(&mut self, rust: Rust, steps: u64) {
        let floor = self.floor(rust);
        for _ in 0..steps {
            let gap = u64::from(self.practice.saturating_sub(floor));
            if gap == 0 {
                return;
            }
            let loss = (gap * u64::from(rust.step_per_mille)).div_ceil(1000);
            self.practice -= loss as u32;
        }
    }
}
