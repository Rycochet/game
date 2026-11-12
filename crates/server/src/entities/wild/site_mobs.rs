//! A site's own mobs in the wilderness: those living on a site's first level
//! set into it are about there too, made with the block their home is in and
//! kept with their site (its first level's `LevelSave`), never in both
//! places at once.

use shared::mob::Spawn;
use shared::wilderness::BlockPos;
use std::collections::BTreeSet;

use super::super::{Mob, dressed};
use crate::ServerWorld;
use crate::ids::new_id;
use crate::storage::SiteId;

/// The mobs of a site's first level whose homes are in `block`, made in the
/// frame `shift` cells along from where it is round the planet; none while
/// the site's own visit has them, or for those dead.
pub(super) fn site_mobs(world: &mut ServerWorld, block: BlockPos, shift: i64) -> Vec<Mob> {
    let wild = &world.wilderness.wild;
    let (bx, by, columns, rows) = wild.block_bounds(block);
    let chunk = wild.chunk_of(bx, by);
    let Some(wild_chunk) = crate::sites::wild_chunk(world, chunk) else {
        return Vec::new();
    };
    let inside =
        |&(x, y): &(i64, i64)| (bx..bx + columns).contains(&x) && (by..by + rows).contains(&y);
    let plan: Vec<Spawn> = wild_chunk
        .spawns
        .iter()
        .filter(|spawn| inside(&spawn.position))
        .cloned()
        .collect();
    let id = SiteId::Poi(chunk);
    if plan.is_empty() || world.populations.0.contains_key(&(id, 0)) {
        return Vec::new();
    }
    let made_at = world.made_now();
    let saved = world.site_mut(id).levels.entry(0).or_default();
    plan.into_iter()
        .filter(|spawn| !saved.dead.contains(&spawn.number))
        .map(|spawn| {
            let mob = *saved.mobs.entry(spawn.number).or_insert_with(new_id);
            let record = saved.records.entry(mob).or_default();
            let carrying = record
                .gear
                .get_or_insert_with(|| dressed(spawn.kind, &spawn.gear, spawn.coins, made_at))
                .clone();
            let memory = record.memory.clone();
            let position = (spawn.position.0 + shift, spawn.position.1);
            let spawn = Spawn { position, ..spawn };
            Mob {
                memory,
                kept_in: Some(id),
                ..Mob::new(mob, &spawn, &carrying, 0)
            }
        })
        .collect()
}

/// A site's own visit is making its first level's mobs: those resting in
/// the wilderness (kept with a block nobody holds) come home to it, and the
/// numbers of those still about in an area are given, for it to leave out.
pub(crate) fn take_site_mobs_back(world: &mut ServerWorld, id: SiteId) -> BTreeSet<u32> {
    let maps = &mut world.wilderness;
    for (mobs, _) in maps.areas.kept.values_mut() {
        for mob in mobs.extract_if(.., |mob| mob.kept_in == Some(id)) {
            let home = maps.wild.block_of(mob.home.0, mob.home.1);
            if let Some(count) = maps.areas.living.get_mut(&home) {
                *count = count.saturating_sub(1);
                if *count == 0 {
                    maps.areas.living.remove(&home);
                }
            }
        }
    }
    maps.areas.kept.retain(|_, (mobs, _)| !mobs.is_empty());
    world
        .populations
        .0
        .iter()
        .filter(|((site, _), _)| *site == SiteId::Wilderness)
        .flat_map(|(_, population)| &population.mobs)
        .filter(|mob| mob.kept_in == Some(id))
        .map(|mob| mob.number)
        .collect()
}
