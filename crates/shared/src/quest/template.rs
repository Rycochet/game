//! Templates: the steps a hook makes, sometimes with a second step at the
//! same poi, in series or in parallel. A poi's boss and unique item come
//! from the poi's own seed, as everyone sees the same ones.

use super::area::Candidate;
use super::hook::slain_with_weights;
use super::plan::Plan;
use super::world::size;
use super::{Alternative, Hook, Step, Target, Way};
use crate::poi::Poi;
use crate::rng::{Rng, Seed};
use crate::skill::SkillId;
use crate::story::{Boss, Goal, ItemCategory, Pet};

/// Percent of quests with a second step at the same poi.
const PAIRED_PERCENT: u64 = 35;
/// Percent of escorts that are of a pet rather than a relative.
const PET_PERCENT: u64 = 60;

/// The plan for a quest on `main`, perhaps pairing it with one of `others`
/// at the same poi. `town` is the giver's town.
pub(super) fn plan(town: &Poi, main: &Candidate, others: &[Candidate], rng: &mut Rng) -> Plan {
    let poi = main.poi;
    let mut plan = Plan {
        place: Some(poi),
        ..Plan::default()
    };
    let paired = (rng.range_u64(0, 100) < PAIRED_PERCENT)
        .then(|| partner(main, others, rng))
        .flatten();
    let mut after = Vec::new();
    if let Some(first) = paired {
        add(&mut plan, town, first, Vec::new(), rng);
        // The guards before their boss, or the way down before a find, in
        // series; anything else alongside.
        if main.hook == Hook::Boss || first.hook == Hook::Explore {
            after.push(0);
        }
    }
    add(&mut plan, town, *main, after, rng);
    plan
}

/// A second step for `main`: counted kills, or exploring, at the same poi.
fn partner(main: &Candidate, others: &[Candidate], rng: &mut Rng) -> Option<Candidate> {
    if !matches!(main.hook, Hook::Boss | Hook::Item | Hook::Explore) {
        return None;
    }
    let choices: Vec<(usize, u64)> = (others.iter().enumerate())
        .filter(|(_, c)| c.poi.chunk == main.poi.chunk && !c.is_wilds())
        .filter(|(_, c)| match c.hook {
            Hook::Kills(_) => true,
            Hook::Explore => main.hook == Hook::Item,
            _ => false,
        })
        .map(|(i, c)| (i, c.weight))
        .collect();
    (!choices.is_empty()).then(|| others[rng.pick_weighted(&choices)])
}

/// Adds the step `candidate` makes, and what the story needs for it.
fn add(plan: &mut Plan, town: &Poi, candidate: Candidate, after: Vec<u8>, rng: &mut Rng) {
    let poi = candidate.poi;
    let target = if candidate.is_wilds() {
        Target::Wilds { chunk: poi.chunk }
    } else {
        Target::Poi {
            chunk: poi.chunk,
            hook: candidate.hook,
        }
    };
    let bigger = size(town.kind).unwrap_or(0) as u64;
    let mut step = Step {
        goal: Goal::Visit,
        target,
        count: None,
        after,
        alternatives: Vec::new(),
    };
    match candidate.hook {
        Hook::Boss => {
            step.goal = Goal::Slay;
            let boss = boss_of(&poi);
            plan.creatures.get_or_insert((boss.kind, 1));
            plan.boss = Some(boss);
            step.alternatives = alternative(Way::Intimidate, "social/intimidating");
        }
        Hook::Item => {
            step.goal = Goal::Find;
            plan.item = Some(item_of(&poi));
        }
        Hook::Escortee => {
            step.goal = Goal::Escort;
            if rng.range_u64(0, 100) < PET_PERCENT {
                plan.pet = Some(Pet::new(Seed::new(rng.next_u64())));
                step.alternatives = vec![Alternative {
                    way: Way::Carry,
                    skill: None,
                }];
                step.alternatives
                    .extend(alternative(Way::Tame, "social/animals"));
            }
        }
        Hook::Kills(kind) => {
            step.goal = Goal::Slay;
            let count = rng.range_u64(3 + bigger, 7 + 2 * bigger) as u32;
            step.count = Some(count);
            plan.creatures = Some((kind, count));
        }
        Hook::Explore => step.goal = Goal::Explore,
        Hook::Visit => step.goal = Goal::Visit,
        Hook::Gather => {
            step.goal = Goal::Gather;
            step.count = Some(rng.range_u64(5, 11) as u32);
            plan.item = Some(ItemCategory::Herb);
        }
    }
    if candidate.hook.is_exclusive() {
        plan.claims.push((poi.chunk, candidate.hook));
    }
    plan.steps.push(step);
}

/// A way of doing a step, if its skill is in the tree.
fn alternative(way: Way, skill: &str) -> Vec<Alternative> {
    SkillId::find(skill)
        .map(|skill| Alternative {
            way,
            skill: Some(skill),
        })
        .into_iter()
        .collect()
}

/// A poi's quest boss: the same whoever asks, from the poi's own seed.
pub(super) fn boss_of(poi: &Poi) -> Boss {
    let seed = poi.seed.derive("quest boss");
    let kinds = slain_with_weights(poi.kind);
    let kind = seed.rng().pick_weighted(&kinds);
    Boss::new(kind, seed.derive("boss"))
}

/// What a poi's unique quest item is: something someone left behind.
pub(super) fn item_of(poi: &Poi) -> ItemCategory {
    let keepsakes: Vec<(ItemCategory, u64)> = (ItemCategory::ALL.iter())
        .filter(|c| c.is_keepsake())
        .map(|&c| (c, 1))
        .collect();
    poi.seed
        .derive("quest item")
        .rng()
        .pick_weighted(&keepsakes)
}
