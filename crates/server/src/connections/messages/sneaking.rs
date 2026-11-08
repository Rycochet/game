//! Sneaking (`Ability::Sneak`, a toggle): on, a player's steps on a level take
//! at least `energy::SNEAK_TICKS_PER_CELL` each; and the pace they have kept
//! lately, which Sneaking is learned at only when slow enough.

use shared::energy::{SNEAK_TICKS_PER_CELL, sneak_step_energy};
use shared::protocol::ServerMessage;
use std::collections::VecDeque;

use crate::connections::Client;

/// How far back the pace is measured: a minute of action ticks.
const PACE_TICKS: u64 = 60 * 20;

/// Whether a player sneaks, and when their recent steps on levels were taken.
#[derive(Clone, Debug, Default)]
pub(crate) struct Sneak {
    pub(crate) on: bool,
    /// The ticks of steps within `PACE_TICKS`, oldest first, with the cells each
    /// covered.
    steps: VecDeque<(u64, u32)>,
}

impl Sneak {
    /// Notes a step of `cells` at `now`: whether the pace over the last minute,
    /// this step included, is a sneaking one (no quicker than a cell every
    /// `SNEAK_TICKS_PER_CELL`).
    pub(crate) fn stepped(&mut self, now: u64, cells: u32) -> bool {
        while self
            .steps
            .front()
            .is_some_and(|&(at, _)| at + PACE_TICKS < now)
        {
            self.steps.pop_front();
        }
        // The time since the oldest step kept, against the cells walked since.
        let since = self.steps.front().map_or(u64::MAX, |&(at, _)| now - at);
        let walked: u32 = self.steps.iter().map(|&(_, cells)| cells).sum();
        self.steps.push_back((now, cells));
        since >= u64::from(walked) * u64::from(SNEAK_TICKS_PER_CELL)
    }
}

impl Client {
    /// Turns sneaking on or off, telling the client. `false` if it has gone.
    pub(crate) fn toggle_sneak(&mut self) -> bool {
        let sneak = &mut self.learning.sneak;
        sneak.on = !sneak.on;
        let on = sneak.on;
        bevy::log::debug!("Sneaking {}", if on { "on" } else { "off" });
        self.link.send(&ServerMessage::Sneaking(on))
    }

    /// What a step on a level costing `cost` and covering `cells` takes,
    /// sneaking or not.
    pub(super) fn sneak_cost(&self, cost: u32, cells: u32) -> u32 {
        if self.learning.sneak.on {
            sneak_step_energy(cost, cells, self.speed)
        } else {
            cost
        }
    }
}

#[cfg(test)]
mod tests;
