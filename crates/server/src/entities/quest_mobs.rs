//! Quests' mobs on site levels (owner): a poi's lone quest boss, on its
//! deepest level from the start, seen by everyone, coming back (Claude) for
//! anyone on its step or who hasn't killed it; and the extras a "Slay" step
//! brings, so a quester finds enough of the kind for what they still need.

use shared::creature::CreatureKind;
use shared::dungeon::Level;
use shared::item::Quality;
use shared::mob::{Memory, Spawn, chosen_spawns};
use shared::poi::Poi;
use shared::quest::{boss_of, boss_tier};
use shared::rng::Seed;
use shared::sheet::Attribute;

use super::made::{BOSS_NUMBER, EXTRA_NUMBERS};
use super::{Mob, dressed};
use crate::ServerWorld;
use crate::ids::new_id;
use crate::storage::SiteId;

/// What each step of a boss's tier above Common adds to every attribute.
const TIER_ATTRIBUTES: u16 = 2;

/// The most extras one arrival brings, however many are still needed.
const MOST_EXTRAS: u32 = 8;

/// The quest boss of `poi`'s site, if level `index` (of `levels`) is its
/// deepest and a town's quest asks for it: rolled as a level's own, with the
/// boss's own kind and gender, from the poi's seed.
pub(super) fn boss_spawn(
    world: &mut ServerWorld,
    poi: &Poi,
    (index, levels): (u32, u32),
    level: &Level,
    stairs: &[(i64, i64)],
) -> Option<Spawn> {
    if index + 1 != levels || crate::quests::boss_quest(world, poi.chunk).is_none() {
        return None;
    }
    let boss = boss_of(poi);
    let seed = poi.seed.derive("quest boss").derive("spawn");
    // Clear of the stairs if there is room; on a small level (a pyramid's
    // top), anywhere it can stand.
    let mut spawn = (chosen_spawns(seed, boss.kind, 1, level, stairs).pop())
        .or_else(|| chosen_spawns(seed, boss.kind, 1, level, &[]).pop())?;
    spawn.number = BOSS_NUMBER;
    // Its gender is the story's, with that gender's bonus in place of the one
    // rolled; and each tier above Common is stronger (owner): Claude's first
    // guess, two to every attribute a step.
    let (rolled, own) = (spawn.gender.bonus(), boss.gender.bonus());
    spawn.gender = boss.gender;
    let steps = (boss_tier(poi) as u16).saturating_sub(Quality::Common as u16);
    for which in Attribute::ALL {
        let now = spawn
            .attributes
            .get(which)
            .saturating_sub(rolled.get(which));
        let tiered = now + own.get(which) + TIER_ATTRIBUTES * steps;
        spawn.attributes.set(which, tiered);
    }
    spawn.appearance = spawn.appearance.fitted(boss.gender);
    Some(spawn)
}

impl ServerWorld {
    /// Where the stairs are on a level in use, which what a quest places
    /// keeps away from.
    pub(crate) fn stairs_of(&self, place: (SiteId, u32)) -> Vec<(i64, i64)> {
        (self.populations.0.get(&place)).map_or_else(Vec::new, |p| p.stairs.clone())
    }

    /// Brings a site's quest boss back on level `place`, if it is down there
    /// (Claude, owner: for someone on its step or who hasn't killed it). Not
    /// kept: on the level's next populating it is down again, unless someone
    /// brings it back. Whether it came.
    pub(crate) fn bring_back_boss(&mut self, place: (SiteId, u32), poi: &Poi, levels: u32) -> bool {
        let (site, index) = place;
        let Some(population) = self.populations.0.get(&place) else {
            return false;
        };
        if population.mobs.iter().any(Mob::is_boss) {
            return false;
        }
        let level = population.level.clone();
        let stairs = population.stairs.clone();
        let saved = self.site(site).levels.get(&index);
        if !saved.is_some_and(|saved| saved.dead.contains(&BOSS_NUMBER)) {
            return false;
        }
        let Some(spawn) = boss_spawn(self, poi, (index, levels), &level, &stairs) else {
            return false;
        };
        // Its id is a new one, and what it carries rolled again with it.
        let id = new_id();
        let made_at = self.made_now();
        let carrying = dressed(spawn.kind, &spawn.gear, spawn.coins, made_at);
        let saved = self.site_mut(site).levels.entry(index).or_default();
        saved.mobs.insert(BOSS_NUMBER, id);
        saved.records.entry(id).or_default().gear = Some(carrying.clone());
        let mob = Mob {
            memory: Memory::default(),
            ..Mob::new(id, &spawn, &carrying, index)
        };
        if let Some(population) = self.populations.0.get_mut(&place) {
            population.mobs.push(mob);
        }
        true
    }

    /// Brings extras of `kind` onto level `place` until the site's loaded
    /// levels have `wanted` of them alive (owner: for a quester's "Slay"
    /// step). They carry nothing and are never kept. How many came.
    pub(crate) fn top_up(
        &mut self,
        place: (SiteId, u32),
        poi: &Poi,
        kind: CreatureKind,
        wanted: u32,
    ) -> u32 {
        let alive: usize = (self.populations.0.iter())
            .filter(|((site, _), _)| *site == place.0)
            .map(|(_, population)| population.mobs.iter().filter(|m| m.kind == kind).count())
            .sum();
        let short = wanted.saturating_sub(alive as u32).min(MOST_EXTRAS);
        if short == 0 {
            return 0;
        }
        let tick = self.action_clock.ticks();
        let made_at = self.made_now();
        let Some(population) = self.populations.0.get_mut(&place) else {
            return 0;
        };
        let stairs = population.stairs.clone();
        let seed = Seed::new(poi.seed.derive("quest extras").at(&[tick as i64]));
        let spawns = chosen_spawns(seed, kind, short as u8, &population.level, &stairs);
        let mut number = (population.mobs.iter().map(|mob| mob.number))
            .filter(|&n| (EXTRA_NUMBERS..BOSS_NUMBER).contains(&n))
            .max()
            .map_or(EXTRA_NUMBERS, |n| n + 1);
        let mut came = 0;
        for mut spawn in spawns {
            if population
                .mobs
                .iter()
                .any(|mob| mob.position == spawn.position)
            {
                continue;
            }
            spawn.number = number;
            number += 1;
            spawn.gear.clear();
            spawn.coins = 0;
            let carrying = dressed(spawn.kind, &spawn.gear, 0, made_at);
            population
                .mobs
                .push(Mob::new(new_id(), &spawn, &carrying, place.1));
            came += 1;
        }
        came
    }
}
