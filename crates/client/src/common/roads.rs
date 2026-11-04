//! The roads the server has told the client of.

use std::collections::BTreeMap;

use shared::road::{Road, RoadLink};

/// Roads between towns, as the server sends them (it works them out, as towns
/// come from the secret seed). Kept for as long as the client is in the world.
#[derive(Default)]
pub struct KnownRoads {
    roads: BTreeMap<RoadLink, Road>,
    /// Goes up with each road added, so what is drawn from them can tell it is
    /// out of date.
    changes: u32,
}

impl KnownRoads {
    pub fn insert(&mut self, road: Road) {
        self.roads.insert(road.link, road);
        self.changes = self.changes.wrapping_add(1);
    }

    /// Every road, in no order that matters.
    pub fn iter(&self) -> impl Iterator<Item = &Road> {
        self.roads.values()
    }

    /// How many roads have been added, wrapping: different after any change.
    pub fn changes(&self) -> u32 {
        self.changes
    }
}
