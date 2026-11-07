//! `Windows`: at most one practice per thing every 5 seconds, the largest that came
//! in that time counting (owner, 2026-11-07), so a quick weapon learns no faster than
//! a slow one. Kept per skill for practice, and per kind of creature for the blocks
//! spent fighting each (`sheet::Foes`).

use std::collections::BTreeMap;

/// How long one window lasts, in action ticks (20 a second): 5 seconds.
pub const WINDOW_TICKS: u64 = 100;

/// One window: when it closes, and how much it has given so far.
#[derive(Clone, Copy, Debug)]
struct Window {
    ends: u64,
    given: u32,
}

/// The open windows, by what they are for. Not saved: a reconnection starts afresh.
#[derive(Clone, Debug)]
pub struct Windows<K>(BTreeMap<K, Window>);

impl<K> Default for Windows<K> {
    fn default() -> Self {
        Self(BTreeMap::new())
    }
}

impl<K: Ord + Copy> Windows<K> {
    /// How much of `amount` to give `key` now, at action tick `now`. With no window
    /// open, all of it, and a window opens; inside one, only what it is above the
    /// largest given so far, so the window gives its largest single amount.
    pub fn take(&mut self, key: K, amount: u32, now: u64) -> u32 {
        self.0.retain(|_, window| window.ends > now);
        let window = self.0.entry(key).or_insert(Window {
            ends: now + WINDOW_TICKS,
            given: 0,
        });
        let more = amount.saturating_sub(window.given);
        window.given += more;
        more
    }
}
