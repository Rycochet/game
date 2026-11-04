//! The game's and the server's clocks, the night light, and herbs' growth shift.

use bevy::prelude::*;
use shared::world::{MINUTES_PER_HOUR, WorldTime};

/// In-game time of day, in whole hours (`0..24`), as the server says
/// (`connection`). Only changes once per hour, so systems can redraw on
/// `is_changed()`.
#[derive(Resource, Default, PartialEq)]
pub struct GameClock {
    pub hour: u32,
}

/// Stages added to how far herbs have grown by the clock, to look at them
/// (debug menu, dev tools): only what this client draws.
#[derive(Resource, Default)]
pub struct HerbGrowthShift(pub u32);

/// The server's clock as well as the client can tell, and the world's time
/// worked out from it: game minutes since the world began. Unlike `GameClock`
/// it changes every frame, unless paused.
#[derive(Resource, Default)]
pub struct ServerClock {
    /// The world's time, from `seconds` and `time` (`ServerClock::set`).
    pub minutes: f64,
    /// The server's clock: real seconds it has run, kept in step by pings.
    pub seconds: f64,
    /// How the world's time goes by the server's clock.
    pub time: WorldTime,
    /// The server says the clock has stopped. Game time (`Time`) stops with it.
    pub paused: bool,
    /// The server would stop it if asked (`ClientMessage::SetPaused`).
    pub can_pause: bool,
    /// How long the last `Ping` took to come back, in milliseconds.
    pub round_trip_ms: Option<f64>,
    /// The server is in this process: `round_trip_ms` says nothing about a
    /// network, only how long it waited for the server's tick, so it shows as a dash.
    pub local_server: bool,
}

impl ServerClock {
    /// The server's clock reads `seconds`, and the world's time goes by it as
    /// `time` says.
    pub fn set(&mut self, seconds: f64, time: WorldTime) {
        self.seconds = seconds;
        self.time = time;
        self.minutes = time.minutes(seconds);
    }

    /// The same in game hours, which the tide, weather and sky go by.
    pub fn hours(&self) -> f64 {
        self.minutes / MINUTES_PER_HOUR
    }
}

/// The dull light everywhere, however dark it gets, as a fraction of full light.
pub const NIGHT_LIGHT: f32 = 0.1;
