//! A cave's plan: its levels, worked out from the seed.

use super::super::SiteLevel;
use super::super::grid::{chance, roll};
use super::{CaveEntrance, CaveOptions, open_skylights};
use crate::rng::Seed;

#[derive(Clone, Debug)]
pub struct Plan {
    pub(super) seed: Seed,
    pub(super) options: CaveOptions,
    pub(super) levels: u32,
    pub(super) opening: bool,
}

impl Plan {
    pub fn new(options: &CaveOptions, seed: Seed) -> Self {
        let mut rng = seed.derive("cave").rng();
        let levels = roll(&mut rng, options.levels.0 as i64, options.levels.1 as i64).max(1) as u32;
        let opening = match options.entrance {
            CaveEntrance::Hole => false,
            CaveEntrance::Opening => true,
            CaveEntrance::Either => chance(&mut rng, 50),
        };
        Self {
            seed,
            options: *options,
            levels,
            opening,
        }
    }

    pub fn levels(&self) -> u32 {
        self.levels
    }

    pub fn level(&self, index: u32) -> SiteLevel {
        let mut made = self.without_skylights(index);
        if index == 0 && !self.options.doors {
            let seed = self.seed.derive("skylight");
            open_skylights(&mut made.level, seed, &made.stairs);
        }
        made
    }
}
