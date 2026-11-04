//! The Hall of Worlds game server: the authoritative simulation. See
//! `docs/context/server/server.md`.
//!
//! Native only. It runs either as the standalone `hall-of-worlds-server`
//! binary (dedicated / Docker) or embedded in a native client for single
//! player, on its own thread with its own Bevy world. The web client never
//! includes it. Clients talk to it only through messages
//! (`shared::protocol`), even when embedded. It saves its world, sites,
//! characters (`storage`) and accounts (`accounts`) in its own data folder.

mod accounts;
mod clock;
mod connections;
pub mod debug_api;
mod embedded;
mod entities;
mod environment;
mod ground;
mod ids;
mod latency;
mod lets_encrypt;
mod network;
mod save_queue;
mod saves;
mod sites;
mod stack_limits;
mod storage;
mod tls;
mod world_map;

use bevy::app::ScheduleRunnerPlugin;
use bevy::prelude::*;
use bevy::time::common_conditions::on_timer;
use shared::world::{WORLD_MAP_VERSION, WorldSeed};
use std::path::PathBuf;
use std::time::Duration;

pub use accounts::new_guest_token;
pub use connections::Connections;
pub use embedded::{EmbeddedServer, start_embedded};
pub use environment::{log_plugin, option_on};
pub use network::{NetworkPlugin, OnlinePlugin, check_health};
pub use shared::protocol::CharacterId;
pub use storage::{DATA_FOLDER_VARIABLE, default_data_folder};
pub use tls::{CertificateFacts, CertificateSource, Security, certificate_facts};

use save_queue::SaveQueue;
use storage::{SAVE_FORMAT, Storage, WorldSave};

// Split into files (`cargo xtask lint` keeps files small).
mod server_world;

pub use server_world::ServerWorld;

/// Simulation ticks per second.
pub const TICKS_PER_SECOND: f64 = shared::energy::ACTION_TICKS_PER_SECOND;

/// The world every server runs until worlds can be created and chosen.
pub const DEFAULT_WORLD_SEED: u64 = 12345;

/// How often what has changed is saved (it is also saved when a client
/// leaves).
const SAVE_EVERY: Duration = Duration::from_secs(5);

/// The server sends what is visible from every walkable cell within this
/// many steps of the player in a site, so the client can work out sight
/// itself and keep up while moving (more covers more latency, for more
/// sent). A candidate for a `ServerOptions` setting.
const LEVEL_PRELOAD_STEPS: i64 = 3;

pub struct ServerPlugin {
    pub world_seed: WorldSeed,
    pub options: ServerOptions,
    /// The data folder (`default_data_folder`), or `None` to save nothing.
    pub data: Option<PathBuf>,
}

/// How a server is run.
#[derive(Resource, Clone, Copy, Debug, Default)]
pub struct ServerOptions {
    /// Whether clients may use developer tools (`DevEnterSite`): only for
    /// the embedded single-player server.
    pub dev_tools: bool,
    /// Whether the server stops (after saving) once its last client has
    /// left: the embedded server, whose one client is the game.
    pub exit_when_alone: bool,
    /// Whether clients may play without an account (`HOW_GUESTS`).
    pub guests: bool,
    /// Whether a client may stop the world's clock: only the embedded server,
    /// whose one client is the only one playing. Separate from `dev_tools`, so
    /// a test server with those on can't be stopped by anyone connected.
    pub pausable: bool,
    /// Whether an embedded server is open to other players: then its dev
    /// tools are its own player's alone.
    pub online: bool,
    /// The most players at once, 0 for no limit (`HOW_PLAYER_LIMIT`).
    pub player_limit: u16,
}

impl ServerOptions {
    /// Whether this client may use dev tools: on a server that allows them, its
    /// host (the embedded server's own player) always, others unless it is an
    /// embedded server open to other players.
    pub(crate) fn dev_tools_for(&self, host: bool) -> bool {
        self.dev_tools && (host || !self.online)
    }
}

/// What the server calls itself (`HOW_SERVER_NAME`, or Server Options).
#[derive(Resource, Clone, Debug, PartialEq, Eq)]
pub struct ServerName(pub String);

impl Default for ServerName {
    fn default() -> Self {
        Self("Hall of Worlds".into())
    }
}

impl Plugin for ServerPlugin {
    fn build(&self, app: &mut App) {
        if let Some(data) = &self.data {
            saves::set_aside_old_saves(data)
                .unwrap_or_else(|error| panic!("The server can't set aside old saves: {error}"));
        }
        // One save thread for the world and the accounts.
        let queue = self.data.as_deref().map(SaveQueue::in_folder);
        let storage = match &queue {
            Some(queue) => Storage::new(queue.clone(), self.world_seed.id()),
            None => Storage::none(),
        };
        let world = ServerWorld::open(self.world_seed, storage)
            .unwrap_or_else(|error| panic!("The server can't open its world: {error}"));
        app.insert_resource(world)
            .insert_resource(self.options)
            .init_resource::<ServerName>()
            .insert_resource(Connections::new(match queue {
                Some(queue) => accounts::Accounts::new(queue),
                None => accounts::Accounts::default(),
            }))
            .add_systems(Startup, announce)
            .add_systems(
                Update,
                (
                    advance_action_clock,
                    connections::handle_messages,
                    connections::run_mobs,
                    connections::sync_entities,
                    connections::sync_ground,
                    connections::tell_the_time,
                    connections::save_changes.run_if(on_timer(SAVE_EVERY)),
                    exit_when_alone.run_if(|options: Res<ServerOptions>| options.exit_when_alone),
                )
                    .chain(),
            );
        #[cfg(feature = "debug-api")]
        debug_api::add_server_methods(app, self.data.clone());
    }
}

/// Moves the action clock on by real time, so that a slow frame doesn't slow
/// everyone down, unless the world's clock is paused.
fn advance_action_clock(time: Res<Time>, world: Res<ServerWorld>, mut owed: Local<f64>) {
    if world.clock.is_paused() {
        return;
    }
    *owed += time.delta_secs_f64() * shared::energy::ACTION_TICKS_PER_SECOND;
    let whole = owed.floor();
    *owed -= whole;
    world.action_clock.advance(whole as u64);
}

#[cfg(test)]
mod action_clock_tests;

/// A headless server (no window) ticking `TICKS_PER_SECOND` times a second,
/// saving in `data` (if any). Add a log plugin if nothing else in the
/// process has one.
pub fn headless_app(world_seed: WorldSeed, options: ServerOptions, data: Option<PathBuf>) -> App {
    let mut app = App::new();
    app.add_plugins(
        MinimalPlugins.set(ScheduleRunnerPlugin::run_loop(Duration::from_secs_f64(
            1.0 / TICKS_PER_SECOND,
        ))),
    )
    .add_plugins(ServerPlugin {
        world_seed,
        options,
        data,
    });
    app
}

/// Saves `world.ron`.
fn save_world(storage: &Storage, seed: WorldSeed, clock: f64, action_ticks: u64) {
    storage.save_world(&WorldSave {
        format: SAVE_FORMAT,
        seed,
        world_map_version: WORLD_MAP_VERSION,
        clock,
        action_ticks,
    });
}

/// Stops the server once nobody is connected (each client was saved as it
/// left).
fn exit_when_alone(connections: Res<Connections>, mut exit: MessageWriter<AppExit>) {
    if connections.is_empty() {
        exit.write(AppExit::Success);
    }
}

fn announce(world: Res<ServerWorld>) {
    info!(
        "Hall of Worlds server {} started, world {:?}",
        env!("CARGO_PKG_VERSION"),
        world.seed.id()
    );
}
