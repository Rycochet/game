//! The standalone Hall of Worlds server: a headless Bevy app (no window)
//! running `ServerPlugin` at a fixed tick rate, saving in its data folder
//! (`HALL_OF_WORLDS_SERVER_DATA`, or the user's data folder).

use bevy::log::LogPlugin;
use server::{DEFAULT_WORLD_SEED, ServerOptions, default_data_folder, headless_app};
use shared::world::WorldSeed;

fn main() {
    let data = default_data_folder();
    let mut app = headless_app(
        WorldSeed::new(DEFAULT_WORLD_SEED),
        ServerOptions {
            dev_tools: false,
            exit_when_alone: false,
        },
        Some(data.clone()),
    );
    app.add_plugins(LogPlugin::default());
    bevy::log::info!("Saving in {}", data.display());
    app.run();
}
