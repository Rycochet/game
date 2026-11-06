//! Who lives on each level: populating levels, moving mobs aside, stairs taken and saving them.

use shared::creature::CreatureKind;
use shared::dungeon::line_is_clear;
use shared::energy::Energy;
use shared::mob::{Memory, scent_strength, spawns};
use shared::poi::Poi;
use shared::protocol::CharacterId;
use shared::sheet::{Feeling, Sheet};
use shared::site::SiteLevel;
use std::collections::{BTreeMap, BTreeSet};

use super::scent::Went;
use super::{Doing, Mob, Population, StairsTaken, dressed, nearest_free};
use crate::ServerWorld;
use crate::ids::new_id;
use crate::storage::{SavedMob, SiteId};

impl ServerWorld {
    /// Makes sure a level about to be used has its mobs. Each keeps the id it was
    /// first given, saved with the site.
    pub(crate) fn populate(&mut self, id: SiteId, poi: Poi, index: u32, level: &SiteLevel) {
        if self.populations.0.contains_key(&(id, index)) {
            return;
        }
        let stairs: Vec<_> = level.stairs.iter().map(|s| (s.x, s.y)).collect();
        let plan = spawns(poi.seed, index, poi.kind, &level.level, &stairs);
        let self_tick = self.action_clock.ticks();
        let made_at = self.made_now();
        let saved = self.site_mut(id).levels.entry(index).or_default();
        let unsettled = std::mem::take(&mut saved.unsettled);
        let mut scents = std::mem::take(&mut saved.scents);
        scents.sweep(self_tick);
        let records = saved.records.clone();
        let dead = saved.dead.clone();
        let mut due = BTreeSet::new();
        let tick = self_tick;
        let mobs = plan
            .into_iter()
            .filter(|spawn| !dead.contains(&spawn.number))
            .map(|spawn| {
                let carried_on = unsettled.get(&spawn.number);
                let id = *saved.mobs.entry(spawn.number).or_insert_with(new_id);
                let carrying = saved
                    .records
                    .entry(id)
                    .or_default()
                    .gear
                    .get_or_insert_with(|| dressed(spawn.kind, &spawn.gear, made_at))
                    .clone();
                if carried_on.is_some() {
                    due.insert((tick, id));
                }
                Mob {
                    id,
                    kind: spawn.kind,
                    appearance: spawn.appearance,
                    position: carried_on.map_or(spawn.position, |saved| saved.position),
                    home: spawn.position,
                    doing: carried_on.map_or(Doing::Idle, |saved| saved.doing),
                    target: None,
                    number: spawn.number,
                    memory: records
                        .get(&id)
                        .map_or_else(Memory::default, |r| r.memory.clone()),
                    home_level: index,
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
        let population = Population {
            mobs,
            level: level.level.clone(),
            due,
            away: 0,
            empty_since: None,
            scents,
            last_at: BTreeMap::new(),
        };
        self.populations.0.insert((id, index), population);
    }

    /// Whether a mob on this cell won't let this player past.
    pub(crate) fn enemy_at(
        &self,
        place: (SiteId, u32),
        cell: (i64, i64),
        kind: CreatureKind,
        character: Option<&CharacterId>,
    ) -> bool {
        self.populations.0.get(&place).is_some_and(|population| {
            population.mobs.iter().any(|mob| {
                mob.position == cell && mob.feeling(kind, character) == Feeling::Unfriendly
            })
        })
    }

    /// A player has stepped from `from` to `to`: a mob there takes their place.
    pub(crate) fn swap_mob(&mut self, place: (SiteId, u32), to: (i64, i64), from: (i64, i64)) {
        if let Some(population) = self.populations.0.get_mut(&place) {
            for mob in &mut population.mobs {
                if mob.position == to {
                    mob.position = from;
                }
            }
        }
    }

    /// A player has arrived on this cell, not by a step: a mob there moves to the
    /// nearest free cell it can stand on.
    pub(crate) fn move_mob_aside(&mut self, place: (SiteId, u32), cell: (i64, i64)) {
        let Some(population) = self.populations.0.get_mut(&place) else {
            return;
        };
        let Some(index) = population.mobs.iter().position(|mob| mob.position == cell) else {
            return;
        };
        let taken = |spot: (i64, i64)| population.mobs.iter().any(|mob| mob.position == spot);
        if let Some(spot) = nearest_free(&population.level, cell, taken) {
            population.mobs[index].position = spot;
        }
    }

    /// Mobs after someone who has just taken stairs, and saw, follow if they can.
    pub(crate) fn saw_stairs_taken(&mut self, taken: StairsTaken) {
        let place = (taken.site, taken.from_level);
        let now = self.action_clock.ticks();
        let Some(population) = self.populations.0.get_mut(&place) else {
            return;
        };
        // Their trail leads up or down here (nobody in god mode is in `last_at`).
        if let Some((_, strength)) = population.last_at.remove(&taken.who) {
            let went = Went::Stairs {
                to_level: taken.to_level,
                arrives: taken.arrives,
            };
            population
                .scents
                .lay(taken.who, taken.stairs, went, strength, now);
        }
        for mob in &mut population.mobs {
            if let Doing::Going { after, gap, .. } = mob.doing
                && after == taken.who
                && mob.kind.def().uses_stairs
                && line_is_clear(&population.level, mob.position, taken.stairs)
            {
                mob.doing = Doing::Following {
                    after,
                    stairs: taken.stairs,
                    to_level: taken.to_level,
                    arrives: taken.arrives,
                    gap,
                };
            }
        }
    }

    /// Notes in the sites' saves which mobs aren't at rest, to carry on after a
    /// restart. Those on other levels start again from home.
    pub(crate) fn save_mobs(&mut self) {
        let mut notes = Vec::new();
        for (&(site, level), population) in &self.populations.0 {
            // A wild area's number isn't a level, and only changes are kept
            // there: its mobs and scent start again after a restart.
            if site == SiteId::Wilderness {
                continue;
            }
            let unsettled: BTreeMap<u32, SavedMob> = population
                .mobs
                .iter()
                .filter(|mob| {
                    mob.doing != Doing::Idle && mob.home_level == level && mob.way_back.is_empty()
                })
                .map(|mob| {
                    let saved = SavedMob {
                        position: mob.position,
                        doing: mob.doing,
                    };
                    (mob.number, saved)
                })
                .collect();
            let scents = population.scents.clone();
            notes.push((site, level, unsettled, scents));
        }
        for (site, level, unsettled, scents) in notes {
            let known = self.site(site).levels.get(&level);
            let same =
                known.is_none_or(|saved| saved.unsettled == unsettled && saved.scents == scents);
            let nothing = unsettled.is_empty() && scents.is_empty();
            if !same || (known.is_none() && !nothing) {
                let saved = self.site_mut(site).levels.entry(level).or_default();
                saved.unsettled = unsettled;
                saved.scents = scents;
            }
        }
    }
}
