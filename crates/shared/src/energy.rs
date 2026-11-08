//! Time for doing things. See `docs/context/shared/energy.md`.
//!
//! A creature gains its speed in energy every tick. It can act with 0 or more,
//! and acting spends the action's cost, which can take it below 0: a dearer action
//! means a longer wait for the next, and a faster creature acts more often.
//!
//! Nothing touches a creature between actions. Only its energy and the tick that
//! was correct at are stored, and the formula gives it at any other tick, so a
//! thousand creatures with nothing to do cost nothing, and time away only refills
//! them.

use serde::{Deserialize, Serialize};

use crate::movement::step_cost;

/// How many ticks of action time pass in a second. The server's ticks, and what
/// the client counts for itself.
pub const ACTION_TICKS_PER_SECOND: f64 = 20.0;

/// What an ordinary creature gains per tick. With these costs, it walks 4 cells a
/// second in a site and 60 points a second on the world map.
pub const NORMAL_SPEED: u32 = 210;

/// Other actions in a site (stairs, dressing, moving things between containers) cost
/// this many times a world map point's energy, the scales being so different.
pub const SITE_SCALE: u32 = 5;

/// A step in a site costs this many times a world map step: 4 cells a second at
/// `NORMAL_SPEED`, slow enough that something coming round a corner can be seen
/// before it is beside you. Tweak to change the pace inside sites.
pub const SITE_STEP_SCALE: u32 = 15;

/// Cells a second walked in a site at `NORMAL_SPEED` (what the client paces click
/// travel by).
pub const SITE_CELLS_PER_SECOND: f64 = NORMAL_SPEED as f64 * ACTION_TICKS_PER_SECOND
    / (crate::movement::STRAIGHT_COST * SITE_STEP_SCALE) as f64;

/// Taking stairs or going into or out of a site: two straight steps' worth.
pub const CHANGE_MAP_COST: u32 = 2 * 70 * SITE_SCALE;

/// How much a step in a site costs: its distance.
pub fn site_step_cost(from: (i64, i64), to: (i64, i64)) -> u32 {
    step_cost(from, to) * SITE_STEP_SCALE
}

/// How much a step on a level costs: its distance, any slope in the
/// wilderness and wading (`movement::level_step_cost`).
pub fn level_step_energy(
    from: (i64, i64),
    to: (i64, i64),
    heights: Option<(i32, i32)>,
    wading: bool,
    gait: crate::movement::Gait,
) -> u32 {
    crate::movement::level_step_cost(from, to, heights, wading, gait) * SITE_STEP_SCALE
}

/// Ticks a cell takes at least while sneaking (`ability::Ability::Sneak`): one
/// every 2 seconds, the pace Sneaking is learned at (owner, 2026-11-08: to tweak
/// once the movement skills are done).
pub const SNEAK_TICKS_PER_CELL: u32 = 2 * ACTION_TICKS_PER_SECOND as u32;

/// What a step on a level costs while sneaking, for someone gaining `speed` a
/// tick: at least `SNEAK_TICKS_PER_CELL` of their energy for each cell it
/// covers, so they go no quicker than that.
pub fn sneak_step_energy(cost: u32, cells: u32, speed: u32) -> u32 {
    cost.max(speed * SNEAK_TICKS_PER_CELL * cells.max(1))
}

/// How much putting on or taking off something this heavy costs: plate takes
/// longer than a hat.
pub fn dressing_cost(grams: u32) -> u32 {
    /// A hat's worth, whatever it weighs.
    const AT_LEAST: u32 = 70 * SITE_SCALE;
    AT_LEAST + grams / 10
}

/// How much moving something this heavy from one container to another costs: 10 kg
/// takes a twelfth of a second at `NORMAL_SPEED`. Nothing is quite free. To depend
/// on stats, later.
pub fn handling_cost(grams: u32) -> u32 {
    const GRAMS_PER_STEP: u64 = 10_000;
    let step = (70 * SITE_SCALE) as u64;
    (grams as u64 * step).div_ceil(GRAMS_PER_STEP) as u32
}

/// How much energy something has, and the tick that was correct at.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Energy {
    energy: i64,
    at: u64,
}

impl Energy {
    /// How much it has at `tick`, at this speed. Stops just under a tick's worth,
    /// so waiting saves nothing up, but fast creatures lose no fractions of a tick.
    pub fn at(&self, tick: u64, speed: u32) -> i64 {
        let gained = tick.saturating_sub(self.at).saturating_mul(speed as u64);
        let gained = i64::try_from(gained).unwrap_or(i64::MAX);
        self.energy
            .saturating_add(gained)
            .min(speed.max(1) as i64 - 1)
    }

    pub fn ready(&self, tick: u64, speed: u32) -> bool {
        self.at(tick, speed) >= 0
    }

    /// Spends this much at `tick`.
    pub fn spend(&mut self, tick: u64, speed: u32, cost: u32) {
        self.energy = self.at(tick, speed) - cost as i64;
        self.at = tick;
    }

    /// Records what it has now, for when the speed is about to change: the time
    /// before now went at this speed.
    pub fn settle(&mut self, tick: u64, speed: u32) {
        self.spend(tick, speed, 0);
    }

    /// Gives back what is still owed beyond `penalty`, as when what it was spent
    /// on is called off before it was done: it can act again after only that
    /// much more.
    pub fn cancel(&mut self, tick: u64, speed: u32, penalty: u32) {
        self.energy = self.at(tick, speed).max(-i64::from(penalty));
        self.at = tick;
    }

    /// The first tick at or after `tick` when it can act, at this speed.
    pub fn ready_at(&self, tick: u64, speed: u32) -> u64 {
        let now = self.at(tick, speed);
        if now >= 0 {
            return tick;
        }
        tick + (-now as u64).div_ceil(speed.max(1) as u64)
    }
}

#[cfg(test)]
mod tests;
