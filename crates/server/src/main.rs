//! The standalone Hall of Worlds server: a headless Bevy app (no window)
//! running `ServerPlugin` at a fixed tick rate.

use std::time::Duration;

use bevy::app::ScheduleRunnerPlugin;
use bevy::log::LogPlugin;
use bevy::prelude::*;
use server::{ServerPlugin, TICKS_PER_SECOND};

fn main() {
    App::new()
        .add_plugins(
            MinimalPlugins.set(ScheduleRunnerPlugin::run_loop(Duration::from_secs_f64(
                1.0 / TICKS_PER_SECOND,
            ))),
        )
        .add_plugins(LogPlugin::default())
        .add_plugins(ServerPlugin)
        .run();
}
