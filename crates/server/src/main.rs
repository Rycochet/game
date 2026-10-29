//! The standalone Hall of Worlds server: a headless Bevy app (no window)
//! running `ServerPlugin` at a fixed tick rate.

use bevy::log::LogPlugin;
use server::{DEFAULT_WORLD_SEED, headless_app};
use shared::world::WorldSeed;

fn main() {
    let mut app = headless_app(WorldSeed::new(DEFAULT_WORLD_SEED));
    app.add_plugins(LogPlugin::default());
    app.run();
}
