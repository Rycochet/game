//! Roads between towns, worked out on a thread of their own, as a long one
//! takes a while, and kept (in memory and in each world's saves) so each is
//! only ever worked out once. One thread for every world the server runs.
//! `connections::send_roads` tells players of them.

use std::collections::BTreeMap;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{Receiver, Sender, channel};
use std::sync::{Arc, Mutex};
use std::thread::{self, JoinHandle};

use bevy::prelude::*;
use shared::poi::PoiMap;
use shared::road::{Road, RoadLink, Roads};
use shared::world::{WorldId, WorldParams, WorldSeed};

use crate::storage::Storage;

/// What the road thread is asked.
enum Ask {
    /// Every road through an area, `(min, max)` in world grid points, of the
    /// world of `seed`, saved in `storage`.
    Area {
        seed: WorldSeed,
        storage: Storage,
        min: (i64, i64),
        max: (i64, i64),
    },
    /// A world has closed: let go of what is known of it, and its saves.
    Forget(WorldId),
}

/// Asks the road thread for the roads through areas, and hands back what it
/// has found, by world. Dropping it stops the thread and waits for it, so the
/// thread's handles on the saves have gone by then: the save queue only
/// finishes writing once its last handle goes, and a server (or a test's
/// world) being closed must save at once.
pub(crate) struct RoadWorker {
    /// `None` only while being dropped (dropping it ends the thread's wait). In
    /// a `Mutex` because a `Sender` can't be shared between threads otherwise.
    asks: Option<Mutex<Sender<Ask>>>,
    /// In a `Mutex` only because a resource must be shareable between threads
    /// and a `Receiver` isn't; only the game ever takes from it.
    found: Mutex<Receiver<(WorldId, Arc<Road>)>>,
    /// Tells the thread to stop before its next road, rather than after the
    /// rest of an area.
    stop: Arc<AtomicBool>,
    /// `None` only while being dropped.
    thread: Option<JoinHandle<()>>,
}

impl RoadWorker {
    pub(crate) fn start() -> Self {
        let (asks, asked) = channel::<Ask>();
        let (tell, found) = channel();
        let stop = Arc::new(AtomicBool::new(false));
        let stopping = stop.clone();
        let thread = thread::Builder::new()
            .name("roads".into())
            .spawn(move || work(&asked, &tell, &stopping))
            .expect("can start the road thread");
        Self {
            asks: Some(Mutex::new(asks)),
            found: Mutex::new(found),
            stop,
            thread: Some(thread),
        }
    }

    /// Asks for every road through an area of the world of `seed`. They come
    /// back from `found` once worked out, the ones already known too, as
    /// whoever asked needs them.
    pub(crate) fn ask(&self, seed: WorldSeed, storage: &Storage, min: (i64, i64), max: (i64, i64)) {
        self.send(Ask::Area {
            seed,
            storage: storage.clone(),
            min,
            max,
        });
    }

    /// Lets go of a world that has closed, and its saves.
    pub(crate) fn forget(&self, world: WorldId) {
        self.send(Ask::Forget(world));
    }

    fn send(&self, ask: Ask) {
        // The thread only stops when this is dropped, so this can't fail.
        if let Some(asks) = &self.asks {
            let _ = asks.lock().expect("never poisoned").send(ask);
        }
    }

    /// The roads found since last time, with the worlds they are in.
    pub(crate) fn found(&self) -> Vec<(WorldId, Arc<Road>)> {
        let found = self
            .found
            .lock()
            .expect("the road thread never panics holding it");
        found.try_iter().collect()
    }
}

/// What the road thread knows of one world.
struct WorldRoads {
    pois: PoiMap,
    roads: Roads,
    storage: Storage,
    known: BTreeMap<RoadLink, Option<Arc<Road>>>,
}

impl WorldRoads {
    fn new(seed: WorldSeed, storage: Storage) -> Self {
        let params = WorldParams::from_seed(seed.terrain());
        Self {
            pois: PoiMap::new(&params, seed.pois()),
            roads: Roads::new(&params),
            storage,
            known: BTreeMap::new(),
        }
    }

    /// The road along `link`: known, saved, or worked out now (and saved).
    fn road(&mut self, link: RoadLink) -> Option<Arc<Road>> {
        if let Some(road) = self.known.get(&link) {
            return road.clone();
        }
        let road = self.storage.load_road(link).unwrap_or_else(|| {
            let road = self.roads.road(&self.pois, link);
            debug!(
                "Worked out the road {link:?}: {} points",
                road.as_ref().map_or(0, |r| r.points.len())
            );
            self.storage.save_road(link, road.clone());
            road
        });
        let road = road.map(Arc::new);
        self.known.insert(link, road.clone());
        road
    }
}

/// The road thread: answers each area asked about with its roads, working out
/// any not known yet.
fn work(asked: &Receiver<Ask>, tell: &Sender<(WorldId, Arc<Road>)>, stop: &AtomicBool) {
    let mut worlds: BTreeMap<WorldId, WorldRoads> = BTreeMap::new();
    while let Ok(ask) = asked.recv() {
        let (seed, storage, min, max) = match ask {
            Ask::Area {
                seed,
                storage,
                min,
                max,
            } => (seed, storage, min, max),
            Ask::Forget(world) => {
                worlds.remove(&world);
                continue;
            }
        };
        let world = seed.id();
        let roads = worlds
            .entry(world)
            .or_insert_with(|| WorldRoads::new(seed, storage));
        let links: Vec<RoadLink> = roads
            .roads
            .links_in_area(&roads.pois, min, max)
            .into_iter()
            .map(|(link, _)| link)
            .collect();
        for link in links {
            if stop.load(Ordering::Relaxed) {
                return;
            }
            if let Some(road) = roads.road(link)
                && tell.send((world, road)).is_err()
            {
                return;
            }
        }
    }
}

impl Drop for RoadWorker {
    fn drop(&mut self) {
        self.stop.store(true, Ordering::Relaxed);
        // With nothing left to ask, the thread's wait for the next area ends.
        self.asks = None;
        if let Some(thread) = self.thread.take()
            && thread.join().is_err()
        {
            error!("The road thread panicked");
        }
    }
}

#[cfg(test)]
mod tests;
