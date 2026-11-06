//! A cave's plan: its levels, worked out from the seed.

use super::super::SiteLevel;
use super::super::grid::{chance, roll};
use super::outcrop::{LEAST_ROCK, raise_outcrop};
use super::{CaveEntrance, CaveOptions, open_skylights};
use crate::rng::Seed;

#[derive(Clone, Debug)]
pub struct Plan {
    pub(super) seed: Seed,
    pub(super) options: CaveOptions,
    pub(super) levels: u32,
    pub(super) opening: bool,
    /// Whether a cave mouth's top level is a hill of rock (`CAVE_VERSION`),
    /// not the whole rectangle (`BOXED_CAVE_VERSION`).
    pub(super) outcrop: bool,
}

impl Plan {
    /// A cave's plan; `outcrop` for the current version (see `outcrop`).
    pub fn new(options: &CaveOptions, seed: Seed, outcrop: bool) -> Self {
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
            outcrop,
        }
    }

    pub fn levels(&self) -> u32 {
        self.levels
    }

    pub fn level(&self, index: u32) -> SiteLevel {
        let (mut made, mouths) = self.dug(index);
        if index == 0 && !self.options.doors {
            let seed = self.seed.derive("skylight");
            open_skylights(&mut made.level, seed, &made.stairs);
        }
        if self.outcrop && !mouths.is_empty() {
            let mut rng = self.seed.derive("outcrop").rng();
            let depth = roll(&mut rng, LEAST_ROCK.0, LEAST_ROCK.1);
            raise_outcrop(&mut made, &mouths, depth);
        }
        made
    }
}
