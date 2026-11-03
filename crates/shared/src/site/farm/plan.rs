//! A farm's plan: its farmhouse and fields.

use super::super::SiteLevel;
use super::super::building::{self};
use super::{FarmOptions, lay_out};
use crate::rng::Seed;

/// A farm's plan. Fields depend on where the house's door is, so its ground
/// floor is made at once and kept.
#[derive(Clone, Debug)]
pub struct Plan {
    house: building::Plan,
    ground: SiteLevel,
}

impl Plan {
    pub fn new(options: &FarmOptions, seed: Seed) -> Self {
        let house = building::Plan::new(&options.house, seed);
        let ground = lay_out(options, seed, &house);
        Self { house, ground }
    }

    pub fn levels(&self) -> u32 {
        self.house.floors()
    }

    pub fn has_basement(&self) -> bool {
        self.house.has_basement()
    }

    pub fn level(&self, index: u32) -> SiteLevel {
        match index {
            0 => self.ground.clone(),
            _ => self.house.level(index),
        }
    }

    /// The house's size, without the ground round it.
    pub fn footprint(&self) -> (i64, i64) {
        self.house.footprint()
    }

    /// How many fields there are, without any pond.
    pub fn fields(&self) -> usize {
        self.ground.fields.len()
    }
}
