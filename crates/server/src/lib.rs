//! The Hall of Worlds game server: the authoritative simulation. See
//! `docs/context/server/server.md`.
//!
//! Native only. It runs either as the standalone `hall-of-worlds-server`
//! binary (dedicated / Docker) or embedded in a native client for single
//! player, on its own thread with its own Bevy world. The web client never
//! includes it. Clients talk to it only through messages
//! (`shared::protocol`), even when embedded.

mod connections;
mod embedded;

use std::time::Duration;

use bevy::app::ScheduleRunnerPlugin;
use bevy::prelude::*;
use shared::protocol::WorldInfo;
use shared::world::{WORLD_MAP_VERSION, WorldSeed};

pub use connections::Connections;
pub use embedded::start_embedded;

/// Simulation ticks per second.
pub const TICKS_PER_SECOND: f64 = 20.0;

/// The world every server runs until worlds can be created and saved.
pub const DEFAULT_WORLD_SEED: u64 = 12345;

pub struct ServerPlugin {
    pub world_seed: WorldSeed,
}

impl Plugin for ServerPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(ServerWorld {
            seed: self.world_seed,
        })
        .init_resource::<Connections>()
        .add_systems(Startup, announce)
        .add_systems(Update, connections::handle_messages);
    }
}

/// A headless server (no window) ticking `TICKS_PER_SECOND` times a second.
/// Add a log plugin if nothing else in the process has one.
pub fn headless_app(world_seed: WorldSeed) -> App {
    let mut app = App::new();
    app.add_plugins(
        MinimalPlugins.set(ScheduleRunnerPlugin::run_loop(Duration::from_secs_f64(
            1.0 / TICKS_PER_SECOND,
        ))),
    )
    .add_plugins(ServerPlugin { world_seed });
    app
}

/// The world this server runs.
#[derive(Resource)]
pub struct ServerWorld {
    pub seed: WorldSeed,
}

impl ServerWorld {
    /// What a joining client is told about the world.
    pub fn info(&self) -> WorldInfo {
        WorldInfo {
            id: self.seed.id(),
            terrain_seed: self.seed.terrain(),
            world_map_version: WORLD_MAP_VERSION,
            features_seed: self.seed.features(),
        }
    }
}

fn announce(world: Res<ServerWorld>) {
    info!(
        "Hall of Worlds server {} started, world {:?}",
        env!("CARGO_PKG_VERSION"),
        world.seed.id()
    );
}
