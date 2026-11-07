//! Who is put in a Dev Area: worked out from its choices alone, like any level's
//! mobs, so nothing about them needs saving.

use super::{DevArea, DevMobs};
use crate::mob::{Spawn, chosen_spawns};
use crate::rng::Seed;
use crate::site::SiteLevel;

impl DevArea {
    /// Its mobs on `level` (its one level), away from the stairs. The plain
    /// layouts have nobody by default.
    pub fn spawns(&self, level: &SiteLevel) -> Vec<Spawn> {
        match self.checked().mobs {
            DevMobs::Default | DevMobs::None => Vec::new(),
            DevMobs::Kind { kind, count } => {
                let clear_of: Vec<_> = level.stairs.iter().map(|s| (s.x, s.y)).collect();
                let seed = Seed::new(self.seed).derive("dev area mobs");
                chosen_spawns(seed, kind, count, &level.level, &clear_of)
            }
        }
    }
}
