//! Rust: skills wearing back towards a floor, by playtime (`docs/context/abilities-skills.md`,
//! "Rust and opposition"). Each node keeps a clock, the playtime before which it can't
//! rust again; disuse, opposition (`opposition.rs`) and dying each take steps, and
//! using a node pushes its own clock on. Integers only: client and server agree.

use super::learned::NOT_SEEN;
use super::opposition::opposed_by;
use super::{SkillId, Skills};
use crate::energy::ACTION_TICKS_PER_SECOND;

/// Action ticks in an hour of playtime.
pub const HOUR: u64 = 3600 * ACTION_TICKS_PER_SECOND as u64;

/// At most one step of rust per node in this much playtime, from disuse and
/// opposition alike (they don't stack). *To balance.*
pub const STEP_EVERY: u64 = HOUR;

/// How a node rusts: written on a node in the tree's draft and passed down to
/// everything under it that doesn't say otherwise.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Rust {
    /// Playtime unused before disuse starts to rust it, or never.
    pub disuse_after: Option<u64>,
    /// How much of what is above the floor one step takes, per mille.
    pub step_per_mille: u32,
    /// Where rust stops, per mille of the best practice reached.
    pub floor_per_mille: u32,
}

impl Rust {
    /// Rusts only by opposition and dying: the top levels' default.
    pub const STEADY: Self = Self {
        disuse_after: None,
        step_per_mille: 10,
        floor_per_mille: 500,
    };

    /// Also rusts after 8 hours unused (Melee: owner, 2026-11-08). *To balance.*
    pub const DISUSED: Self = Self {
        disuse_after: Some(8 * HOUR),
        ..Self::STEADY
    };
}

impl Skills {
    /// A player joining at this playtime: nodes from a save made before rust
    /// count as used now.
    pub fn settle(&mut self, played: u64) {
        for learned in self.0.values_mut() {
            if learned.used == NOT_SEEN {
                learned.used = played;
                learned.clock = played;
            }
        }
    }

    /// This node was used at this playtime: it and every node above it that is
    /// known push their clocks on, and the nodes they oppose take a step of rust
    /// if their clocks have passed. Whether anything rusted.
    pub fn used(&mut self, id: SkillId, played: u64) -> bool {
        for node in id.upwards() {
            if let Some(learned) = self.0.get_mut(&node) {
                learned.used = played;
                learned.clock = learned.clock.max(played + STEP_EVERY);
            }
        }
        let mut rusted = false;
        for target in opposed_by(id) {
            let Some(learned) = self.0.get_mut(&target) else {
                continue;
            };
            if played >= learned.clock {
                let before = learned.practice;
                learned.rust(target.rust(), 1);
                learned.clock = played + STEP_EVERY;
                rusted |= learned.practice != before;
            }
        }
        rusted
    }

    /// Disuse up to this playtime: each node whose rust says so takes a step for
    /// every `STEP_EVERY` since it was last used (after its `disuse_after`) or
    /// last rusted, however often this is asked. Whether anything rusted.
    pub fn rust(&mut self, played: u64) -> bool {
        let mut rusted = false;
        for (&id, learned) in &mut self.0 {
            let rust = id.rust();
            let Some(after) = rust.disuse_after else {
                continue;
            };
            let start = learned.used.saturating_add(after).max(learned.clock);
            if played < start {
                continue;
            }
            let steps = (played - start) / STEP_EVERY + 1;
            let before = learned.practice;
            learned.rust(rust, steps);
            learned.clock = start + steps * STEP_EVERY;
            rusted |= learned.practice != before;
        }
        rusted
    }

    /// Dying: every known node takes the steps this many hours of disuse would,
    /// whatever its clock, which is left alone (Normal 8, Hard 24: owner,
    /// 2026-11-08).
    pub fn rust_for_hours(&mut self, hours: u32) {
        let steps = u64::from(hours) * HOUR / STEP_EVERY;
        for (&id, learned) in &mut self.0 {
            learned.rust(id.rust(), steps);
        }
    }
}
