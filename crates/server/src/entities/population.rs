//! Who lives on each level: populating levels, moving mobs aside, stairs taken and saving them.

use shared::clearance::{Crowding, Other};
use shared::creature::CreatureKind;
use shared::dungeon::line_is_clear;
use shared::mob::{Memory, spawns};
use shared::poi::Poi;
use shared::protocol::CharacterId;
use shared::sheet::Feeling;
use shared::site::SiteLevel;
use shared::spot::Spot;
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
        // A first level set into the wilderness: those about there stay there.
        let about = match id {
            SiteId::Poi(_) if index == 0 => super::wild::take_site_mobs_back(self, id),
            _ => BTreeSet::new(),
        };
        let plan = match id {
            SiteId::DevArea(area) => area.spawns(level),
            _ => spawns(poi.seed, index, poi.kind, &level.level, &stairs),
        };
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
            .filter(|spawn| !dead.contains(&spawn.number) && !about.contains(&spawn.number))
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
                let mut mob = Mob {
                    doing: carried_on.map_or(Doing::Idle, |saved| saved.doing),
                    memory: records
                        .get(&id)
                        .map_or_else(Memory::default, |r| r.memory.clone()),
                    ..Mob::new(id, &spawn, &carrying, index)
                };
                if let Some(saved) = carried_on {
                    // Older saves have only the cell: its middle.
                    mob.place(saved.spot.unwrap_or(Spot::middle_of(saved.position)));
                }
                mob
            })
            .collect();
        let population = Population {
            mobs,
            level: level.level.clone(),
            doors: Some(super::MobDoors {
                poi,
                open: level.level.with_doors_open(),
            }),
            due,
            away: 0,
            empty_since: None,
            scents,
            last_at: BTreeMap::new(),
        };
        self.populations.0.insert((id, index), population);
    }

    /// The mobs about a player on `place`, as far as their moves go
    /// (`clearance::crowd`): enemies block, the rest are squeezed past. Other
    /// players are walked through, so they aren't here.
    pub(crate) fn crowd_round(
        &self,
        place: (SiteId, u32),
        kind: CreatureKind,
        character: Option<&CharacterId>,
    ) -> Vec<Other> {
        let Some(population) = self.populations.0.get(&place) else {
            return Vec::new();
        };
        let crowding = |mob: &Mob| match mob.feeling(kind, character) {
            Feeling::Unfriendly => Crowding::Block,
            _ => Crowding::Squeeze,
        };
        population
            .mobs
            .iter()
            .map(|mob| Other {
                at: mob.spot,
                size: mob.size(),
                crowding: crowding(mob),
            })
            .collect()
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
            population.mobs[index].put(spot);
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
                    mob.doing != Doing::Idle
                        && mob.home_level == level
                        && mob.way_back.is_empty()
                        && !mob.is_dev_made()
                })
                .map(|mob| {
                    let saved = SavedMob {
                        position: mob.position,
                        spot: Some(mob.spot),
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
