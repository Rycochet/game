//! A world's time: worked out from the server's clock, moved or sped up by dev
//! tools, and saved in `world.ron` only when it changes that way.

use shared::world::{ClockRate, WORLD_MAP_VERSION, WorldTime};

use super::ServerWorld;
use crate::storage::{SAVE_FORMAT, WorldSave};

impl ServerWorld {
    /// Game minutes since the world began.
    pub(crate) fn minutes(&self) -> f64 {
        self.time.minutes(self.server_clock.seconds())
    }

    /// The world's time, as clients are told it.
    pub(crate) fn time(&self) -> WorldTime {
        self.time
    }

    /// Moves the world's time to `minutes`, and/or has it run at `rate` from
    /// now; everyone in it is told (`clock_moved`) and it is saved.
    pub(crate) fn set_time(&mut self, minutes: Option<f64>, rate: Option<ClockRate>) {
        let now = self.server_clock.seconds();
        if let Some(rate) = rate {
            self.time = self.time.with_rate(rate, now);
        }
        if let Some(minutes) = minutes {
            self.time = self.time.set(minutes, now);
        }
        self.clock_moved = true;
        self.save_world();
    }

    /// Saves `world.ron`: its time, by the server's clock now.
    pub(crate) fn save_world(&self) {
        self.storage.save_world(&WorldSave {
            format: SAVE_FORMAT,
            seed: self.seed,
            world_map_version: WORLD_MAP_VERSION,
            clock: self.minutes(),
            server_clock: Some(self.server_clock.seconds()),
            rate: self.time.rate,
            action_ticks: self.action_clock.ticks(),
        });
    }
}
