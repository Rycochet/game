//! A world's time, worked out from the server's clock: every world on a server
//! goes by the one clock (real seconds the server has run, stopped only by
//! pausing), each from its own starting point and at its own rate, so one
//! planet's time can be moved or sped up without touching the others.

use serde::{Deserialize, Serialize};

use super::GAME_MINUTE_SECONDS;

/// How fast a world's time runs against the server's clock: `game` minutes
/// for every `real` minutes the server's clock gives. Whole numbers, so the
/// rate itself is exact. `NORMAL` is 1 to 1.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ClockRate {
    pub game: u32,
    pub real: u32,
}

impl ClockRate {
    pub const NORMAL: Self = Self { game: 1, real: 1 };

    /// The rate, unless it would stop time or run it on forever (`real` 0).
    pub fn new(game: u32, real: u32) -> Option<Self> {
        (game > 0 && real > 0).then_some(Self { game, real })
    }

    /// The rate as a number, `game / real`.
    pub fn factor(self) -> f64 {
        f64::from(self.game) / f64::from(self.real)
    }
}

impl Default for ClockRate {
    fn default() -> Self {
        Self::NORMAL
    }
}

/// A world's time: game minutes since the world began at any reading of the
/// server's clock. Sent to clients, who keep the server's clock in step by
/// pings and work the world's time out from it.
#[derive(Clone, Copy, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct WorldTime {
    /// The world's game minutes when the server's clock read 0.
    pub offset: f64,
    pub rate: ClockRate,
}

impl WorldTime {
    /// A world at `minutes` when the server's clock reads `server_seconds`.
    pub fn at(minutes: f64, server_seconds: f64, rate: ClockRate) -> Self {
        Self {
            offset: minutes - server_minutes(server_seconds) * rate.factor(),
            rate,
        }
    }

    /// Game minutes since the world began, when the server's clock reads
    /// `server_seconds`.
    pub fn minutes(self, server_seconds: f64) -> f64 {
        self.offset + server_minutes(server_seconds) * self.rate.factor()
    }

    /// The same time at `server_seconds`, running at another rate from then on:
    /// no jump, only a change of speed.
    pub fn with_rate(self, rate: ClockRate, server_seconds: f64) -> Self {
        Self::at(self.minutes(server_seconds), server_seconds, rate)
    }

    /// Moved to `minutes` at `server_seconds`, at the same rate. Not before the
    /// world began.
    pub fn set(self, minutes: f64, server_seconds: f64) -> Self {
        Self::at(minutes.max(0.0), server_seconds, self.rate)
    }
}

/// The server's clock in minutes at the normal rate.
fn server_minutes(server_seconds: f64) -> f64 {
    server_seconds / GAME_MINUTE_SECONDS as f64
}

#[cfg(test)]
mod tests;
