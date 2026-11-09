//! Doors swinging: when a door's state changes, its leaf turns from where it
//! was drawn to its new pose over `SWING_SECONDS`, quick at first and slowing
//! as it arrives, the level's meshes rebuilt each frame meanwhile. A door seen
//! for the first time, a new level, and everything with Reduce Motion on are
//! drawn in place at once.

use bevy::prelude::*;
use std::collections::BTreeMap;

use super::walls::LeafPose;
use crate::common::{CurrentDungeon, Preferences};
use shared::dungeon::Level;

/// How long a leaf takes to swing to its new pose, in seconds.
pub(super) const SWING_SECONDS: f64 = 0.35;

/// Which level doors are drawn for: its index and rectangle.
type LevelKey = (u32, (i64, i64, i64, i64));

/// One leaf on its way to a new pose.
#[derive(Clone, Copy, Debug, PartialEq)]
struct Moving {
    from: LeafPose,
    to: LeafPose,
    started: f64,
}

/// The level's door leaves as drawn. Marked changed only while one is moving,
/// so a still level costs nothing more.
#[derive(Resource, Default, Debug)]
pub struct DoorSwings {
    /// Leaves part way through swinging, as drawn now: the rest are drawn as
    /// they hang (`LeafPose::of`).
    pub drawn: BTreeMap<(i64, i64), LeafPose>,
    /// Where each leaf is going, as last known.
    hanging: BTreeMap<(i64, i64), LeafPose>,
    moving: BTreeMap<(i64, i64), Moving>,
    /// The level these are for: its index and rectangle.
    level: Option<LevelKey>,
}

impl DoorSwings {
    /// Takes in the level's doors as they hang now (`level_changed`: the
    /// level has changed since last time), then moves leaves on to `now`
    /// (seconds). Whether anything drawn changed.
    pub(super) fn update(
        &mut self,
        (key, level, level_changed): (Option<LevelKey>, &Level, bool),
        now: f64,
        reduce_motion: bool,
    ) -> bool {
        let mut changed = false;
        if key != self.level {
            // Another level: everything as it hangs, at once.
            changed = !self.drawn.is_empty();
            *self = Self {
                level: key,
                ..Self::default()
            };
        }
        if level_changed || self.hanging.is_empty() {
            for (at, _) in level.swings() {
                let to = LeafPose::of(level, at);
                let Some(was) = self.hanging.insert(at, to) else {
                    // First seen: as it hangs.
                    continue;
                };
                if was == to {
                    continue;
                }
                if reduce_motion {
                    changed |= self.moving.remove(&at).is_some();
                    self.drawn.remove(&at);
                    continue;
                }
                let from = self.drawn.get(&at).copied().unwrap_or(was);
                self.moving.insert(
                    at,
                    Moving {
                        from,
                        to,
                        started: now,
                    },
                );
            }
        }
        changed |= !self.moving.is_empty();
        let moving = std::mem::take(&mut self.moving);
        for (at, swing) in moving {
            let share = ((now - swing.started) / SWING_SECONDS) as f32;
            if share >= 1.0 {
                self.drawn.remove(&at);
                continue;
            }
            // Quick at first, slowing as it arrives.
            let eased = 1.0 - (1.0 - share) * (1.0 - share);
            self.drawn
                .insert(at, LeafPose::between(swing.from, swing.to, eased));
            self.moving.insert(at, swing);
        }
        changed
    }
}

/// Every frame, before the meshes are built: leaves swing to the doors' new
/// states.
pub(super) fn swing_doors(
    dungeon: Res<CurrentDungeon>,
    time: Res<Time>,
    preferences: Res<Preferences>,
    mut swings: ResMut<DoorSwings>,
) {
    let key = dungeon
        .info
        .as_ref()
        .map(|info| (info.index, dungeon.level.bounds()));
    let reduce = preferences.video.reduce_motion;
    let now = time.elapsed_secs_f64();
    let level = (key, &dungeon.level, dungeon.is_changed());
    if swings.bypass_change_detection().update(level, now, reduce) {
        swings.set_changed();
    }
}

#[cfg(test)]
mod tests;
