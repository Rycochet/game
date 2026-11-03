//! A town's options and plan: how many buildings it aims for, and generating it.

use super::super::SiteLevel;
use super::super::grid::roll;
use super::Town;
use crate::rng::Seed;

/// How a town is generated: its number of buildings, at most and at least.
/// The poi's kind decides these, so its size is known before any visit.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TownOptions {
    pub buildings: (i64, i64),
}

/// A town's plan: how many buildings it aims for.
#[derive(Clone, Debug)]
pub struct Plan {
    pub(super) seed: Seed,
    target: i64,
}

impl Plan {
    /// How many buildings it aims to have.
    pub fn target(&self) -> i64 {
        self.target
    }

    pub fn new(options: &TownOptions, seed: Seed) -> Self {
        let mut rng = Seed::new(seed.derive("town size").at(&[0])).rng();
        Self {
            seed,
            target: roll(&mut rng, options.buildings.0, options.buildings.1),
        }
    }

    /// Generates the town (its only level).
    pub fn level(&self) -> SiteLevel {
        Town::new(self.seed, self.target).generate()
    }
}
