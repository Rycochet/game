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
mod storage;
mod tls;
mod world_map;

use clock::{ActionClock, Repeating, WorldClock};
use entities::Populations;
use std::collections::{BTreeMap, BTreeSet};
use std::path::PathBuf;
use std::time::Duration;

use bevy::app::ScheduleRunnerPlugin;
use bevy::prelude::*;
use bevy::time::common_conditions::on_timer;
use shared::movement::{WorldConditions, WorldGround};
use shared::poi::{PoiKind, PoiMap};
use shared::protocol::WorldInfo;
use shared::site::Generator;
use shared::surface::Surface;
use shared::world::{
    MINUTES_PER_HOUR, NORMAL_SNOW_LINE, WORLD_MAP_VERSION, WorldParams, WorldSeed,
};

pub use accounts::new_guest_token;
pub use connections::Connections;
pub use embedded::{EmbeddedServer, start_embedded};
pub use environment::{log_plugin, option_on};
pub use network::NetworkPlugin;
pub use shared::protocol::CharacterId;
pub use storage::{DATA_FOLDER_VARIABLE, default_data_folder};
pub use tls::{CertificateFacts, CertificateSource, Security, certificate_facts};

use save_queue::SaveQueue;
use storage::{SAVE_FORMAT, SiteId, SiteSave, Storage, Unsaved, WorldSave};

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
#[derive(Resource, Clone, Copy, Debug)]
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
mod action_clock_tests {
    use std::time::Duration;

    use bevy::ecs::system::RunSystemOnce;
    use shared::world::WorldSeed;

    use super::*;

    fn after_a_second(paused: bool) -> u64 {
        let mut app = World::new();
        let world = ServerWorld::new(WorldSeed::new(1));
        if paused {
            world.clock.pause();
        }
        app.insert_resource(world);
        let mut time = Time::<()>::default();
        time.advance_by(Duration::from_secs(1));
        app.insert_resource(time);
        app.run_system_once(advance_action_clock).unwrap();
        app.resource::<ServerWorld>().action_clock.ticks()
    }

    #[test]
    fn it_goes_by_real_time_and_stops_when_paused() {
        assert_eq!(after_a_second(false), TICKS_PER_SECOND as u64);
        assert_eq!(after_a_second(true), 0);
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

/// How often the clock is saved, in real time. A crash loses this much of it.
const CLOCK_SAVE_EVERY: Duration = Duration::from_secs(10);

/// The world this server runs.
#[derive(Resource)]
pub struct ServerWorld {
    pub seed: WorldSeed,
    pub pois: PoiMap,
    /// The world's width in grid points: `x` wraps at this.
    pub width: i64,
    /// The ground as far as walking goes.
    pub(crate) ground: WorldGround,
    params: WorldParams,
    /// Game minutes since the world began, always running.
    pub(crate) clock: WorldClock,
    /// What creatures' energy goes by.
    pub(crate) action_clock: ActionClock,
    /// The clock's whole hours. The tide follows this, so it moves once an
    /// hour rather than all the time.
    pub(crate) hour: u32,
    /// `clock` has been moved, which everyone playing needs telling.
    pub(crate) clock_moved: bool,
    /// Saves the clock (`None`: nowhere to save it).
    _clock_saver: Option<Repeating>,
    /// Above it everything is frozen (not saved: normal weather on starting).
    pub(crate) snow_line: u32,
    pub(crate) storage: Storage,
    /// The sites entered so far (loaded when first needed).
    sites: BTreeMap<SiteId, SiteSave>,
    unsaved_sites: Unsaved<SiteId>,
    /// Sites whose saves couldn't be reached, so what is in `sites` for them is
    /// blank, and must not be saved over the real one.
    unreachable_sites: BTreeSet<SiteId>,
    /// Who is on the site levels in use.
    pub(crate) populations: Populations,
    /// Sites forgotten since `connections::handle_messages` last looked, for
    /// putting out anyone else in them.
    forgotten: Forgotten,
}

/// Which sites have been forgotten.
#[derive(Default)]
pub(crate) struct Forgotten {
    all: bool,
    sites: BTreeSet<SiteId>,
}

impl Forgotten {
    pub(crate) fn is_empty(&self) -> bool {
        !self.all && self.sites.is_empty()
    }

    pub(crate) fn includes(&self, id: SiteId) -> bool {
        self.all || self.sites.contains(&id)
    }
}

impl ServerWorld {
    /// When and where something made now is made.
    pub(crate) fn made_now(&self) -> shared::item::MadeAt {
        shared::item::MadeAt {
            world: self.seed.id(),
            minute: self.clock.minutes() as u64,
        }
    }

    /// A world that is never saved (tests).
    pub fn new(seed: WorldSeed) -> Self {
        Self::open(seed, Storage::none()).expect("nothing to read")
    }

    /// The world saved in `storage`, or a new one (saved straight away). An
    /// error if the save can't be read or was made by a newer server.
    fn open(seed: WorldSeed, storage: Storage) -> Result<Self, String> {
        let saved = storage.load_world()?;
        let saved_ticks = saved.as_ref().map_or(0, |save| save.action_ticks);
        let clock = match saved {
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
                // Older versions differ only in having no towns (1), or no farms or
                // herbalists' huts (2):
                // their worlds carry on with some pois turned into those.
                if !(1..=WORLD_MAP_VERSION).contains(&save.world_map_version) {
                    return Err(format!(
                        "it was made with world map version {}; this server has {WORLD_MAP_VERSION}",
                        save.world_map_version
                    ));
                }
                save.clock
            }
            None => 0.0,
        };
        let params = WorldParams::from_seed(seed.terrain());
        let world_clock = WorldClock::new(clock);
        let save_clock = world_clock.clone();
        let save_to = storage.clone();
        let action_clock = ActionClock::new(saved_ticks);
        let save_ticks = action_clock.clone();
        let clock_saver = storage.saves().then(|| {
            Repeating::start(CLOCK_SAVE_EVERY, move || {
                save_world(&save_to, seed, save_clock.minutes(), save_ticks.ticks());
            })
        });
        let world = Self {
            pois: PoiMap::new(&params, seed.pois()),
            width: Surface::new(&params).width(),
            ground: WorldGround::new(&params),
            params,
            clock: world_clock,
            action_clock,
            hour: (clock / MINUTES_PER_HOUR) as u32,
            clock_moved: false,
            _clock_saver: clock_saver,
            snow_line: NORMAL_SNOW_LINE,
            seed,
            storage,
            sites: BTreeMap::new(),
            unsaved_sites: Unsaved::default(),
            unreachable_sites: BTreeSet::new(),
            populations: Populations::default(),
            forgotten: Forgotten::default(),
        };
        save_world(&world.storage, seed, clock, saved_ticks);
        Ok(world)
    }

    /// What decides where can be walked now.
    pub(crate) fn conditions(&self) -> WorldConditions {
        WorldConditions {
            water: self.params.water_level(self.hour as f64),
            snow_line: self.snow_line as f64,
        }
    }

    /// Loads what is saved about a site (for a `kind` of poi), or makes it new
    /// (with its generator's current version) if nobody has been in. An error if
    /// its save couldn't be reached. Done on going into a site, where someone can
    /// be told, so that `site` doesn't need to fail.
    pub(crate) fn open_site(&mut self, id: SiteId, kind: PoiKind) -> Result<(), String> {
        if self.sites.contains_key(&id) && !self.unreachable_sites.contains(&id) {
            return Ok(());
        }
        let save = self.storage.load_site(id)?.unwrap_or_else(|| {
            self.unsaved_sites.mark(id);
            SiteSave {
                version: Generator::for_poi(kind).version(),
                ..default()
            }
        });
        self.unreachable_sites.remove(&id);
        self.sites.insert(id, save);
        Ok(())
    }

    /// What is saved about a site. Normally open already; if it isn't and its save
    /// can't be reached, it is blank for now and not saved.
    pub(crate) fn site(&mut self, id: SiteId, kind: PoiKind) -> &mut SiteSave {
        if !self.sites.contains_key(&id)
            && let Err(error) = self.open_site(id, kind)
        {
            error!("Using a blank {id:?}, which won't be saved: {error}");
            self.unreachable_sites.insert(id);
            let save = SiteSave {
                version: Generator::for_poi(kind).version(),
                ..default()
            };
            self.sites.insert(id, save);
        }
        self.sites.get_mut(&id).expect("just made sure")
    }

    /// `site`, to be saved: something about it is changing.
    pub(crate) fn site_mut(&mut self, id: SiteId, kind: PoiKind) -> &mut SiteSave {
        self.unsaved_sites.mark(id);
        self.site(id, kind)
    }

    /// Saves the sites that have changed.
    pub(crate) fn save_sites(&mut self) {
        for id in self.unsaved_sites.take() {
            if let Some(save) = self.sites.get(&id)
                && !self.unreachable_sites.contains(&id)
            {
                self.storage.save_site(id, save);
            }
        }
    }

    /// Forgets a site: it is made anew when next entered, with its generator's
    /// current version and no players' changes.
    pub(crate) fn forget_site(&mut self, id: SiteId) {
        self.sites.remove(&id);
        self.unreachable_sites.remove(&id);
        self.unsaved_sites.unmark(&id);
        self.storage.remove_site(id);
        self.populations.forget_site(id);
        self.forgotten.sites.insert(id);
    }

    /// Forgets every site, as `forget_site`.
    pub(crate) fn forget_sites(&mut self) {
        self.sites.clear();
        self.unreachable_sites.clear();
        self.unsaved_sites.take();
        self.storage.remove_sites();
        self.populations.forget_all();
        self.forgotten.all = true;
    }

    /// The sites forgotten since this was last asked.
    pub(crate) fn take_forgotten(&mut self) -> Forgotten {
        std::mem::take(&mut self.forgotten)
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
