//! Who is put in a Dev Area: worked out from its choices alone, like any level's
//! mobs, so nothing about them needs saving.

use super::{DevArea, DevLayout, DevMobs};
use crate::mob::{Spawn, chosen_spawns, spawns, wild_spawns};
use crate::rng::Seed;
use crate::site::SiteLevel;
use crate::wilderness::{BLOCK_SIZE, BlockPos};

/// How far from the stairs nobody starts, as for a level's own mobs.
const FROM_STAIRS: i64 = 6;

impl DevArea {
    /// Its mobs on `level` (its one level), away from the stairs. By default:
    /// nobody in the plain layouts, the wilderness's packs and herds in a
    /// biome (rolled as for each block of the wilderness the area would fill),
    /// and a building's own.
    pub fn spawns(&self, level: &SiteLevel) -> Vec<Spawn> {
        let area = self.checked();
        let clear_of: Vec<_> = level.stairs.iter().map(|s| (s.x, s.y)).collect();
        let seed = Seed::new(area.seed).derive("dev area mobs");
        match (area.mobs, area.layout) {
            (DevMobs::Kind { kind, count }, _) => {
                chosen_spawns(seed, kind, count, &level.level, &clear_of)
            }
            (DevMobs::None, _) => Vec::new(),
            (DevMobs::Default, DevLayout::Building(kind)) => {
                spawns(Seed::new(area.seed), 0, kind, &level.level, &clear_of)
            }
            (DevMobs::Default, DevLayout::Biome(biome)) => {
                let side = (i64::from(area.size) / BLOCK_SIZE).max(1);
                let mut mobs: Vec<Spawn> = Vec::new();
                for block in 0..side * side {
                    let block = BlockPos { x: block, y: 0 };
                    for spawn in wild_spawns(seed, block, &level.level, biome) {
                        let (x, y) = spawn.position;
                        let away = clear_of
                            .iter()
                            .all(|s| (s.0 - x).abs().max((s.1 - y).abs()) >= FROM_STAIRS);
                        let free = mobs.iter().all(|mob| mob.position != spawn.position);
                        if away && free {
                            mobs.push(spawn);
                        }
                    }
                }
                mobs
            }
            (DevMobs::Default, _) => Vec::new(),
        }
    }
}
