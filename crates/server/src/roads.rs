//! Roads between towns, worked out on a thread of their own, as a long one
//! takes a while, and kept (in memory and in the world's saves) so each is
//! only ever worked out once. `connections::send_roads` tells players of them.

use std::collections::BTreeMap;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{Receiver, Sender, channel};
use std::sync::{Arc, Mutex};
use std::thread::{self, JoinHandle};

use bevy::prelude::*;
use shared::poi::PoiMap;
use shared::road::{Road, RoadLink, Roads};
use shared::world::{WorldParams, WorldSeed};

use crate::storage::Storage;

/// An area of the world map, `(min, max)` in world grid points.
type Area = ((i64, i64), (i64, i64));

/// Asks the road thread for the roads through areas, and hands back what it
/// has found. Dropping it stops the thread and waits for it, so the thread's
/// handle on the saves has gone by then: the save queue only finishes writing
/// once its last handle goes, and a world being closed must save at once.
pub(crate) struct RoadWorker {
    /// `None` only while being dropped (dropping it ends the thread's wait).
    asks: Option<Sender<Area>>,
    /// In a `Mutex` only because a resource must be shareable between threads
    /// and a `Receiver` isn't; only the game ever takes from it.
    found: Mutex<Receiver<Arc<Road>>>,
    /// Tells the thread to stop before its next road, rather than after the
    /// rest of an area.
    stop: Arc<AtomicBool>,
    /// `None` only while being dropped.
    thread: Option<JoinHandle<()>>,
}

impl RoadWorker {
    pub(crate) fn start(seed: WorldSeed, storage: Storage) -> Self {
        let (asks, asked) = channel::<Area>();
        let (tell, found) = channel();
        let stop = Arc::new(AtomicBool::new(false));
        let stopping = stop.clone();
        let thread = thread::Builder::new()
            .name("roads".into())
            .spawn(move || work(seed, storage, &asked, &tell, &stopping))
            .expect("can start the road thread");
        Self {
            asks: Some(asks),
            found: Mutex::new(found),
            stop,
            thread: Some(thread),
        }
    }

    /// Asks for every road through an area. They come back from `found` once
    /// worked out, the ones already known too, as whoever asked needs them.
    pub(crate) fn ask(&self, min: (i64, i64), max: (i64, i64)) {
        // The thread only stops when this is dropped, so this can't fail.
        if let Some(asks) = &self.asks {
            let _ = asks.send((min, max));
        }
    }

    /// The roads found since last time.
    pub(crate) fn found(&self) -> Vec<Arc<Road>> {
        let found = self
            .found
            .lock()
            .expect("the road thread never panics holding it");
        found.try_iter().collect()
    }
}

/// The road thread: answers each area asked about with its roads, working out
/// any not known yet.
fn work(
    seed: WorldSeed,
    storage: Storage,
    asked: &Receiver<Area>,
    tell: &Sender<Arc<Road>>,
    stop: &AtomicBool,
) {
    let params = WorldParams::from_seed(seed.terrain());
    let (pois, roads) = (PoiMap::new(&params, seed.pois()), Roads::new(&params));
    let mut known: BTreeMap<RoadLink, Option<Arc<Road>>> = BTreeMap::new();
    while let Ok((min, max)) = asked.recv() {
        for (link, _) in roads.links_in_area(&pois, min, max) {
            if stop.load(Ordering::Relaxed) {
                return;
            }
            let road = known.entry(link).or_insert_with(|| {
                let road = storage.load_road(link).unwrap_or_else(|| {
                    let road = roads.road(&pois, link);
                    debug!(
                        "Worked out the road {link:?}: {} points",
                        road.as_ref().map_or(0, |r| r.points.len())
                    );
                    storage.save_road(link, road.clone());
                    road
                });
                road.map(Arc::new)
            });
            if let Some(road) = road
                && tell.send(road.clone()).is_err()
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
