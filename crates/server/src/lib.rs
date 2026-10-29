//! The Hall of Worlds game server: the authoritative simulation. See
//! `docs/context/server/server.md`.
//!
//! Native only. It runs either as the standalone `hall-of-worlds-server`
//! binary (dedicated / Docker) or embedded in a native client for single
//! player, on its own thread with its own Bevy world. The web client never
//! includes it. Clients talk to it only through messages
//! (`shared::protocol`), even when embedded. It saves its world, sites and
//! characters in its own data folder (`storage`).

mod connections;
mod embedded;
mod sites;
mod storage;
mod world_map;

use std::collections::BTreeMap;
use std::path::PathBuf;
use std::time::Duration;

use bevy::app::ScheduleRunnerPlugin;
use bevy::prelude::*;
use bevy::time::common_conditions::on_timer;
use shared::feature::{FeatureKind, FeatureMap};
use shared::protocol::WorldInfo;
use shared::site::Generator;
use shared::surface::Surface;
use shared::world::{WORLD_MAP_VERSION, WorldParams, WorldSeed};

pub use connections::Connections;
pub use embedded::{EmbeddedServer, start_embedded};
pub use storage::{CharacterName, DATA_FOLDER_VARIABLE, default_data_folder};

use storage::{SAVE_FORMAT, SiteId, SiteSave, Storage, Unsaved, WorldSave};

/// Simulation ticks per second.
pub const TICKS_PER_SECOND: f64 = 20.0;

/// The world every server runs until worlds can be created and chosen.
pub const DEFAULT_WORLD_SEED: u64 = 12345;

/// How often what has changed is saved (it is also saved when a client
/// leaves).
const SAVE_EVERY: Duration = Duration::from_secs(5);

pub struct ServerPlugin {
    pub world_seed: WorldSeed,
    pub options: ServerOptions,
    /// The data folder (`default_data_folder`), or `None` to save nothing.
    pub data: Option<PathBuf>,
}

/// How a server is run.
#[derive(Resource, Clone, Copy, Debug)]
pub struct ServerOptions {
    /// Whether clients may use developer tools (`DevEnterSite`): only for
    /// the embedded single-player server.
    pub dev_tools: bool,
    /// Whether the server stops (after saving) once its last client has
    /// left: the embedded server, whose one client is the game.
    pub exit_when_alone: bool,
}

impl Plugin for ServerPlugin {
    fn build(&self, app: &mut App) {
        let storage = match &self.data {
            Some(data) => Storage::in_folder(data, self.world_seed.id()),
            None => Storage::none(),
        };
        let world = ServerWorld::open(self.world_seed, storage)
            .unwrap_or_else(|error| panic!("The server can't open its world: {error}"));
        app.insert_resource(world)
            .insert_resource(self.options)
            .init_resource::<Connections>()
            .add_systems(Startup, announce)
            .add_systems(
                Update,
                (
                    connections::handle_messages,
                    connections::save_changes.run_if(on_timer(SAVE_EVERY)),
                    exit_when_alone.run_if(|options: Res<ServerOptions>| options.exit_when_alone),
                )
                    .chain(),
            );
    }
}

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

/// The world this server runs.
#[derive(Resource)]
pub struct ServerWorld {
    pub seed: WorldSeed,
    pub features: FeatureMap,
    /// The world's width in grid points: `x` wraps at this.
    pub width: i64,
    pub(crate) storage: Storage,
    /// The sites entered so far (loaded when first needed).
    sites: BTreeMap<SiteId, SiteSave>,
    unsaved_sites: Unsaved<SiteId>,
}

impl ServerWorld {
    /// A world that is never saved (tests).
    pub fn new(seed: WorldSeed) -> Self {
        Self::open(seed, Storage::none()).expect("nothing to read")
    }

    /// The world saved in `storage`, or a new one (saved straight away). An
    /// error if the save can't be read or was made by a newer server.
    fn open(seed: WorldSeed, storage: Storage) -> Result<Self, String> {
        match storage.load_world()? {
            Some(save) => {
                if save.format > SAVE_FORMAT {
                    return Err(format!(
                        "it was saved in format {}; this server reads up to {SAVE_FORMAT}",
                        save.format
                    ));
                }
                if save.seed != seed {
                    return Err("its saved seed isn't the one asked for".into());
                }
                // Only one world map version exists so far.
                if save.world_map_version != WORLD_MAP_VERSION {
                    return Err(format!(
                        "it was made with world map version {}; this server has {WORLD_MAP_VERSION}",
                        save.world_map_version
                    ));
                }
            }
            None => storage.save_world(&WorldSave {
                format: SAVE_FORMAT,
                seed,
                world_map_version: WORLD_MAP_VERSION,
            }),
        }
        let params = WorldParams::from_seed(seed.terrain());
        Ok(Self {
            features: FeatureMap::new(&params, seed.features()),
            width: Surface::new(&params).width(),
            seed,
            storage,
            sites: BTreeMap::new(),
            unsaved_sites: Unsaved::default(),
        })
    }

    /// What is saved about a site (for a `kind` of feature): loaded, or new
    /// (made with its generator's current version) if nobody has been in.
    pub(crate) fn site(&mut self, id: SiteId, kind: FeatureKind) -> &mut SiteSave {
        if !self.sites.contains_key(&id) {
            let save = self.storage.load_site(id).unwrap_or_else(|| {
                self.unsaved_sites.mark(id);
                SiteSave {
                    version: Generator::for_feature(kind).version(),
                    ..default()
                }
            });
            self.sites.insert(id, save);
        }
        self.sites.get_mut(&id).expect("just made sure")
    }

    /// `site`, to be saved: something about it is changing.
    pub(crate) fn site_mut(&mut self, id: SiteId, kind: FeatureKind) -> &mut SiteSave {
        self.unsaved_sites.mark(id);
        self.site(id, kind)
    }

    /// Saves the sites that have changed.
    pub(crate) fn save_sites(&mut self) {
        for id in self.unsaved_sites.take() {
            if let Some(save) = self.sites.get(&id) {
                self.storage.save_site(id, save);
            }
        }
    }

    /// What a joining client is told about the world.
    pub fn info(&self) -> WorldInfo {
        WorldInfo {
            id: self.seed.id(),
            terrain_seed: self.seed.terrain(),
            world_map_version: WORLD_MAP_VERSION,
        }
    }
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
