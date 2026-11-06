//! An area taking blocks in and letting them go: their mobs made, kept and
//! brought back.

use shared::energy::Energy;
use shared::mob::{Memory, scent_strength, wild_mob_id, wild_spawns};
use shared::rng::Seed;
use shared::sheet::Sheet;
use shared::wilderness::BlockPos;
use std::collections::{BTreeMap, BTreeSet};
use std::time::Instant;

use super::super::{Doing, Mob, Population, dressed};
use super::{AREA_BLOCKS, WILD_BLOCK_KEPT, place, resets, trails};
use crate::ServerWorld;
use crate::storage::SiteId;

/// Centres `area` on `centre` (the block a player at `at` is in): its level
/// is made round them, it takes in the blocks round it nobody else holds, and
/// lets go of the rest, with the mobs standing in them.
pub(super) fn centre(world: &mut ServerWorld, area: u32, centre: BlockPos, at: (i64, i64)) {
    world.wilderness.areas.areas.insert(area, centre);
    let level = crate::sites::area_level(world, at, AREA_BLOCKS);
    let columns = world.wilderness.wild.block_columns();
    let areas = &world.wilderness.areas;
    let wanted: BTreeSet<BlockPos> = (-AREA_BLOCKS..=AREA_BLOCKS)
        .flat_map(|dy| (-AREA_BLOCKS..=AREA_BLOCKS).map(move |dx| (dx, dy)))
        .map(|(dx, dy)| BlockPos {
            x: (centre.x + dx).rem_euclid(columns),
            y: centre.y + dy,
        })
        .filter(|block| areas.owner.get(block).is_none_or(|&owner| owner == area))
        .collect();
    let held: BTreeSet<BlockPos> = areas
        .owner
        .iter()
        .filter(|&(_, &owner)| owner == area)
        .map(|(&block, _)| block)
        .collect();
    let population = world
        .populations
        .0
        .entry(place(area))
        .or_insert_with(|| empty(level.clone()));
    population.level = level;
    // Those standing outside what it holds now are kept with their block.
    let wild = &world.wilderness.wild;
    let (staying, leaving): (Vec<Mob>, Vec<Mob>) = std::mem::take(&mut population.mobs)
        .into_iter()
        .partition(|mob| wanted.contains(&wild.block_of(mob.position.0, mob.position.1)));
    population.mobs = staying;
    population
        .due
        .retain(|(_, id)| population.mobs.iter().any(|mob| mob.id == *id));
    keep(world, leaving);
    for &block in held.difference(&wanted) {
        world.wilderness.areas.owner.remove(&block);
        resets::been_near(world, block);
    }
    let seed = world.seed.pois().derive("wild mobs");
    for &block in wanted.difference(&held) {
        world.wilderness.areas.owner.insert(block, area);
        let (mobs, new) = match world.wilderness.areas.kept.remove(&block) {
            Some((mobs, _)) => (mobs, false),
            None => (spawned(world, seed, block, at), true),
        };
        let tick = world.action_clock.ticks();
        let Some(population) = world.populations.0.get_mut(&place(area)) else {
            return;
        };
        // Made new: they have been about lately, and left their scent.
        if new {
            trails::lay_trails(population, seed, &mobs, tick);
        }
        for mob in mobs {
            // A mob kept while chasing carries on as it was.
            if mob.doing != Doing::Idle {
                population.due.insert((tick, mob.id));
            }
            population.mobs.push(mob);
        }
    }
}

/// Lets an area go: everyone in it is kept with the block they stand in.
pub(super) fn close(world: &mut ServerWorld, area: u32) {
    world.wilderness.areas.areas.remove(&area);
    let held: Vec<BlockPos> = world
        .wilderness
        .areas
        .owner
        .iter()
        .filter(|&(_, &owner)| owner == area)
        .map(|(&block, _)| block)
        .collect();
    for block in held {
        world.wilderness.areas.owner.remove(&block);
        resets::been_near(world, block);
    }
    if let Some(population) = world.populations.0.remove(&place(area)) {
        keep(world, population.mobs);
    }
}

/// Mobs let go of by an area, kept with the block each stands in, from now.
fn keep(world: &mut ServerWorld, mobs: Vec<Mob>) {
    let now = Instant::now();
    let maps = &mut world.wilderness;
    for mob in mobs {
        let block = maps.wild.block_of(mob.position.0, mob.position.1);
        let kept = maps
            .areas
            .kept
            .entry(block)
            .or_insert_with(|| (Vec::new(), now));
        kept.0.push(mob);
        kept.1 = now;
    }
}

/// Lets go for good of blocks kept longer than `WILD_BLOCK_KEPT`: their mobs'
/// homes may make them again.
pub(super) fn forget_old(world: &mut ServerWorld, now: Instant) {
    let maps = &mut world.wilderness;
    let old: Vec<BlockPos> = maps
        .areas
        .kept
        .iter()
        .filter(|(_, (_, since))| now.duration_since(*since) >= WILD_BLOCK_KEPT)
        .map(|(&block, _)| block)
        .collect();
    for block in old {
        let Some((mobs, _)) = maps.areas.kept.remove(&block) else {
            continue;
        };
        for mob in mobs {
            let home = maps.wild.block_of(mob.home.0, mob.home.1);
            if let Some(count) = maps.areas.living.get_mut(&home) {
                *count = count.saturating_sub(1);
                if *count == 0 {
                    maps.areas.living.remove(&home);
                }
            }
        }
    }
}

/// A block's own mobs, made new (in the frame of a player at `at`), unless
/// some of them are about already or have died (and it hasn't reset since).
fn spawned(world: &mut ServerWorld, seed: Seed, block: BlockPos, at: (i64, i64)) -> Vec<Mob> {
    if world.wilderness.areas.living.contains_key(&block) {
        return Vec::new();
    }
    let biome = world.wilderness.wild.biome_of(block);
    let width = world.wilderness.wild.width();
    let plan = wild_spawns(seed, block, world.wilderness.block(block), biome);
    resets::reset_if_due(world, seed, block, &plan);
    let made_at = world.made_now();
    let saved = world.site(SiteId::Wilderness).levels.get(&0);
    let dead = saved.map(|level| level.dead.clone()).unwrap_or_default();
    let records: BTreeMap<_, _> = saved.map(|level| level.records.clone()).unwrap_or_default();
    // Into the frame of the player the area is round.
    let turns = (at.0 - block.x * shared::wilderness::BLOCK_SIZE + width / 2).div_euclid(width);
    let mobs: Vec<Mob> = plan
        .into_iter()
        .filter(|spawn| !dead.contains(&spawn.number))
        .map(|spawn| {
            let id = wild_mob_id(seed, spawn.number);
            let record = records.get(&id);
            let carrying = record
                .and_then(|record| record.gear.clone())
                .unwrap_or_else(|| dressed(spawn.kind, &spawn.gear, made_at));
            let position = (spawn.position.0 + turns * width, spawn.position.1);
            Mob {
                id,
                kind: spawn.kind,
                appearance: spawn.appearance,
                position,
                home: position,
                doing: Doing::Idle,
                number: spawn.number,
                memory: record.map_or_else(Memory::default, |r| r.memory.clone()),
                home_level: 0,
                way_back: Vec::new(),
                energy: Energy::default(),
                speed: Sheet {
                    attributes: spawn.attributes,
                    ..Sheet::new(spawn.kind)
                }
                .speed(&carrying),
                shown: carrying.shown(),
                scent: scent_strength(&carrying),
            }
        })
        .collect();
    if !mobs.is_empty() {
        world
            .wilderness
            .areas
            .living
            .insert(block, mobs.len() as u32);
    }
    mobs
}

/// A new area's population, with no one in it yet.
fn empty(level: shared::dungeon::Level) -> Population {
    Population {
        mobs: Vec::new(),
        level,
        due: Default::default(),
        away: 0,
        empty_since: None,
        scents: Default::default(),
        last_at: BTreeMap::new(),
    }
}
