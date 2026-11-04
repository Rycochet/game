//! Which world map steps run along a road, for what they cost.

use std::collections::HashMap;

use super::{Road, RoadKind};

/// A step by its two ends, the smaller first, `x` wrapped round the planet.
type StepKey = ((i64, i64), (i64, i64));

/// Every step along the roads added, by its two ends (either way round), with
/// the largest kind of road it is part of. Both sides keep one: the server of
/// every road it has worked out, a client of the roads it has been sent.
#[derive(Clone, Debug, Default)]
pub struct RoadSteps {
    /// The world's width, as `x` wraps at it (0: no wrapping, for tests).
    width: i64,
    steps: HashMap<StepKey, RoadKind>,
}

impl RoadSteps {
    pub fn new(world_width: i64) -> Self {
        Self {
            width: world_width,
            steps: HashMap::new(),
        }
    }

    /// Adds a road's steps; where it shares one with a road already added, the
    /// larger kind counts.
    pub fn add(&mut self, road: &Road) {
        for pair in road.points.windows(2) {
            let key = self.key(pair[0], pair[1]);
            let kind = self.steps.entry(key).or_insert(road.kind);
            *kind = (*kind).max(road.kind);
        }
    }

    /// The road a step from one point to the next runs along, if any: both
    /// points must be on it, one after the other, so crossing a road doesn't
    /// count.
    pub fn kind_of(&self, from: (i64, i64), to: (i64, i64)) -> Option<RoadKind> {
        self.steps.get(&self.key(from, to)).copied()
    }

    /// Whether any steps have been added.
    pub fn is_empty(&self) -> bool {
        self.steps.is_empty()
    }

    /// A step's name, the same either way round and on any copy of the planet.
    fn key(&self, a: (i64, i64), b: (i64, i64)) -> StepKey {
        let wrap = |(x, y): (i64, i64)| match self.width {
            0 => (x, y),
            width => (x.rem_euclid(width), y),
        };
        let (a, b) = (wrap(a), wrap(b));
        (a.min(b), a.max(b))
    }
}
