//! What every world on a server shares: the clocks and the road thread.

use std::sync::Arc;

use super::ServerWorld;
use crate::clock::{ActionClock, ServerClock};
use crate::roads::RoadWorker;

/// What every world on a server shares: the clocks they go by and the road
/// thread. A world opened on its own (tests) has its own.
#[derive(Clone)]
pub(crate) struct Shared {
    pub clock: ServerClock,
    pub action_clock: ActionClock,
    pub roads: Arc<RoadWorker>,
}

impl Shared {
    /// Clocks at these readings, and a road thread.
    pub fn new(seconds: f64, action_ticks: u64) -> Self {
        Self {
            clock: ServerClock::new(seconds),
            action_clock: ActionClock::new(action_ticks),
            roads: Arc::new(RoadWorker::start()),
        }
    }
}

impl ServerWorld {
    /// The clocks and road thread it goes by, to share with other worlds.
    pub(crate) fn shared(&self) -> Shared {
        Shared {
            clock: self.server_clock.clone(),
            action_clock: self.action_clock.clone(),
            roads: self.roads.clone(),
        }
    }

    /// Asks the road thread for every road through an area of this world.
    pub(crate) fn ask_roads(&self, min: (i64, i64), max: (i64, i64)) {
        self.roads.ask(self.seed, &self.storage, min, max);
    }

    /// The roads the road thread has found since last asked, for any world.
    pub(crate) fn roads_found(&self) -> Vec<(shared::world::WorldId, Arc<shared::road::Road>)> {
        self.roads.found()
    }
}
