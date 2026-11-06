//! Blocks resetting: a block whose mobs have something saved (one died, or
//! was given something) remembers when someone was last near, and once
//! nobody has been for `WILD_RESET` its mobs are made again as new.

use shared::mob::{Spawn, wild_mob_id};
use shared::rng::Seed;
use shared::wilderness::BlockPos;

use crate::ServerWorld;
use crate::storage::{LevelSave, SiteId, now};

/// How long a block nobody has been near keeps what happened to its mobs:
/// an hour (Claude's guess, as sites have none yet).
pub(crate) const WILD_RESET: u64 = 60 * 60;

/// The wilderness's saved level, which holds every wild mob's changes.
fn saved(world: &mut ServerWorld) -> &mut LevelSave {
    world
        .site_mut(SiteId::Wilderness)
        .levels
        .entry(0)
        .or_default()
}

/// Something has happened to a mob whose home is at `home`: its block now
/// has something to reset.
pub(crate) fn touched(world: &mut ServerWorld, home: (i64, i64)) {
    let block = world.wilderness.wild.block_of(home.0, home.1);
    saved(world).wild_blocks.insert((block.x, block.y), now());
}

/// Someone has just been near this block: a block with something to reset
/// starts waiting again from now.
pub(super) fn been_near(world: &mut ServerWorld, block: BlockPos) {
    let has = |world: &mut ServerWorld| {
        let level = world.site(SiteId::Wilderness).levels.get(&0);
        level.is_some_and(|level| level.wild_blocks.contains_key(&(block.x, block.y)))
    };
    if has(world) {
        saved(world).wild_blocks.insert((block.x, block.y), now());
    }
}

/// Before a block's mobs are made: if nobody has been near it for
/// `WILD_RESET`, forgets who of them died and what they knew and had.
pub(super) fn reset_if_due(world: &mut ServerWorld, seed: Seed, block: BlockPos, plan: &[Spawn]) {
    let key = (block.x, block.y);
    let due = |level: &LevelSave| {
        level
            .wild_blocks
            .get(&key)
            .is_some_and(|&near| now().saturating_sub(near) >= WILD_RESET)
    };
    let level = world.site(SiteId::Wilderness).levels.get(&0);
    if !level.is_some_and(due) {
        return;
    }
    let level = saved(world);
    for spawn in plan {
        level.dead.remove(&spawn.number);
        level.records.remove(&wild_mob_id(seed, spawn.number));
    }
    level.wild_blocks.remove(&key);
}
