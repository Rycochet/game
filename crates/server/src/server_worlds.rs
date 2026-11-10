//! `ServerWorlds`: every world the server has open, the clocks they share, and
//! opening and closing them. The home world (`ServerPlugin`'s seed), where new
//! characters arrive, is always open; any other is opened when someone needs
//! it and closed once nobody has been in it for `WORLD_KEPT_FOR`.

use bevy::prelude::*;
use shared::world::{WorldId, WorldSeed};
use std::collections::{BTreeMap, BTreeSet};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::{Duration, Instant};

use super::clock::{ActionClock, Repeating, ServerClock};
use super::server_world::{ServerWorld, Shared};
use super::storage::{SAVE_FORMAT, ServerSave, Storage};

/// How often the server's clocks are saved, in real time. A crash loses this
/// much of them.
const CLOCK_SAVE_EVERY: Duration = Duration::from_secs(10);

/// How long a world stays open with nobody in it and nothing going on.
pub(crate) const WORLD_KEPT_FOR: Duration = Duration::from_secs(5 * 60);

/// The worlds this server has open.
#[derive(Resource)]
pub struct ServerWorlds {
    home: WorldId,
    /// By id: a `BTreeMap` keeps them in order, so they are always gone
    /// through the same way.
    open: BTreeMap<WorldId, ServerWorld>,
    /// Since when each world but the home has had nobody in it and nothing
    /// going on.
    empty_since: BTreeMap<WorldId, Instant>,
    shared: Shared,
    /// The home world's saves (others' are the same, `for_world`).
    storage: Storage,
    /// Whether the server is locked down, as saved with the clocks (the game
    /// goes by `ServerOptions::lockdown`).
    lockdown: Arc<AtomicBool>,
    /// Saves the server's clocks (`None`: nowhere to save them).
    _clock_saver: Option<Repeating>,
}

impl ServerWorlds {
    /// The server's clocks as saved, and its home world, saved in `storage`
    /// (or new). An error if either can't be read.
    pub(crate) fn start(home: WorldSeed, storage: Storage) -> Result<Self, String> {
        let (seconds, ticks, lockdown) = match storage.load_server()? {
            Some(save) => (save.clock, save.action_ticks, save.lockdown),
            // From before the server had clocks of its own: its one world kept
            // the action clock.
            None => (
                0.0,
                storage.load_world()?.map_or(0, |save| save.action_ticks),
                false,
            ),
        };
        let shared = Shared::new(seconds, ticks);
        let world = ServerWorld::open_in(home, storage.clone(), &shared)?;
        let mut worlds = Self::with_home(world);
        worlds.storage = storage;
        worlds.lockdown.store(lockdown, Ordering::Relaxed);
        worlds.save_server();
        worlds._clock_saver = worlds.storage.saves().then(|| {
            let (storage, shared) = (worlds.storage.clone(), worlds.shared.clone());
            let lockdown = worlds.lockdown.clone();
            Repeating::start(CLOCK_SAVE_EVERY, move || {
                let lockdown = lockdown.load(Ordering::Relaxed);
                save_server(&storage, &shared.clock, &shared.action_clock, lockdown);
            })
        });
        Ok(worlds)
    }

    /// `world` as the home, open already, going by its clocks (tests).
    pub fn with_home(world: ServerWorld) -> Self {
        let home = world.seed.id();
        Self {
            home,
            shared: world.shared(),
            storage: world.storage.clone(),
            open: BTreeMap::from([(home, world)]),
            empty_since: BTreeMap::new(),
            lockdown: Arc::default(),
            _clock_saver: None,
        }
    }

    /// The home world back (tests).
    #[cfg(test)]
    pub(crate) fn into_home(mut self) -> ServerWorld {
        self.open.remove(&self.home).expect("always open")
    }

    pub(crate) fn home_id(&self) -> WorldId {
        self.home
    }

    pub fn home(&self) -> &ServerWorld {
        &self.open[&self.home]
    }

    pub fn home_mut(&mut self) -> &mut ServerWorld {
        self.open.get_mut(&self.home).expect("always open")
    }

    /// The server's clock, which every world's time goes by.
    pub(crate) fn clock(&self) -> &ServerClock {
        &self.shared.clock
    }

    pub(crate) fn action_clock(&self) -> &ActionClock {
        &self.shared.action_clock
    }

    #[cfg(any(test, feature = "debug-api"))]
    pub(crate) fn get(&self, id: WorldId) -> Option<&ServerWorld> {
        self.open.get(&id)
    }

    pub(crate) fn get_mut(&mut self, id: WorldId) -> Option<&mut ServerWorld> {
        self.open.get_mut(&id)
    }

    /// The world someone is in, or the home before they are in one. Nobody is
    /// ever in a world that has closed.
    pub(crate) fn of(&mut self, world: Option<WorldId>) -> &mut ServerWorld {
        let id = world
            .filter(|id| self.open.contains_key(id))
            .unwrap_or(self.home);
        self.open
            .get_mut(&id)
            .expect("just checked, and the home is always open")
    }

    /// Every open world, in order.
    pub(crate) fn iter(&self) -> impl Iterator<Item = (WorldId, &ServerWorld)> {
        self.open.iter().map(|(id, world)| (*id, world))
    }

    pub(crate) fn iter_mut(&mut self) -> impl Iterator<Item = (WorldId, &mut ServerWorld)> {
        self.open.iter_mut().map(|(id, world)| (*id, world))
    }

    /// Opens the world of `seed`, saved or new, if it isn't open. An error if
    /// its save can't be read or used.
    pub(crate) fn open(&mut self, seed: WorldSeed) -> Result<WorldId, String> {
        let id = seed.id();
        if !self.open.contains_key(&id) {
            let world = ServerWorld::open_in(seed, self.storage.for_world(id), &self.shared)?;
            info!("Opened world {:?}", id.0);
            self.open.insert(id, world);
        }
        Ok(id)
    }

    /// Opens a world someone's save says they are in, from its own save:
    /// `None` if there is none it could run. An error only if its save
    /// couldn't be reached, as it may be fine.
    pub(crate) fn open_saved(&mut self, id: WorldId) -> Result<Option<WorldId>, String> {
        if self.open.contains_key(&id) {
            return Ok(Some(id));
        }
        let Some(seed) = self.storage.for_world(id).saved_world_seed()? else {
            warn!("World {:?} has no save it could run", id.0);
            return Ok(None);
        };
        match self.open(seed) {
            Ok(id) => Ok(Some(id)),
            Err(error) => {
                error!("World {:?} can't be opened: {error}", id.0);
                Ok(None)
            }
        }
    }

    /// Closes every world but the home that has had nobody in it (`in_use`
    /// lists those with someone) and nothing going on for `WORLD_KEPT_FOR`,
    /// saving it first.
    pub(crate) fn close_idle(&mut self, in_use: &BTreeSet<WorldId>) {
        let now = Instant::now();
        let mut closing = Vec::new();
        for (&id, world) in &self.open {
            if id == self.home || in_use.contains(&id) || !world.populations.0.is_empty() {
                self.empty_since.remove(&id);
                continue;
            }
            let since = *self.empty_since.entry(id).or_insert(now);
            if now.duration_since(since) >= WORLD_KEPT_FOR {
                closing.push(id);
            }
        }
        for id in closing {
            self.close(id);
        }
    }

    /// Saves a world and closes it.
    fn close(&mut self, id: WorldId) {
        let Some(mut world) = self.open.remove(&id) else {
            return;
        };
        self.empty_since.remove(&id);
        world.save_mobs();
        world.save_sites();
        world.save_world();
        self.shared.roads.forget(id);
        info!("Closed world {:?}: nobody has been in it for a while", id.0);
    }

    /// Saves what has changed of every world's mobs and sites, and the server's
    /// clocks.
    pub(crate) fn save_worlds(&mut self) {
        for world in self.open.values_mut() {
            world.save_mobs();
            world.save_sites();
        }
        self.save_server();
    }

    /// Saves the server's clocks now.
    pub(crate) fn save_server(&self) {
        let lockdown = self.saved_lockdown();
        save_server(
            &self.storage,
            &self.shared.clock,
            &self.shared.action_clock,
            lockdown,
        );
    }

    /// Whether the server was locked down when it last saved (or since).
    pub(crate) fn saved_lockdown(&self) -> bool {
        self.lockdown.load(Ordering::Relaxed)
    }

    /// Keeps whether the server is locked down, saving it at once.
    pub(crate) fn keep_lockdown(&self, on: bool) {
        self.lockdown.store(on, Ordering::Relaxed);
        self.save_server();
    }
}

/// Saves `server.ron`.
fn save_server(storage: &Storage, clock: &ServerClock, action_clock: &ActionClock, lockdown: bool) {
    storage.save_server(&ServerSave {
        format: SAVE_FORMAT,
        clock: clock.seconds(),
        action_ticks: action_clock.ticks(),
        lockdown,
    });
}

#[cfg(test)]
mod tests;
