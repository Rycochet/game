//! A plan made into a quest: its reward, what it does to alignment, and its
//! story told.

use super::givers::household;
use super::plan::Plan;
use super::reward::reward;
use super::telling::{Tale, tell_plan};
use super::{Quest, QuestId, QuestKind, QuestWorld, Requirement};
use crate::alignment::Deed;
use crate::poi::Poi;
use crate::rng::{Rng, Seed};
use crate::story::{Goal, Story};

/// Everything about a quest but its plan.
pub(super) struct Making<'a> {
    pub id: QuestId,
    pub kind: QuestKind,
    pub building: u16,
    pub member: u32,
    pub requirements: Vec<Requirement>,
    /// Its place in its chain (0 first).
    pub chain: u8,
    pub earlier: Option<&'a Story>,
    pub dark: bool,
    pub continues: bool,
    pub text_seed: Seed,
}

pub(super) fn make(
    world: &QuestWorld,
    town: &Poi,
    plan: &Plan,
    making: Making,
    rng: &mut Rng,
) -> Quest {
    let daily = making.kind == QuestKind::Daily;
    let (reward, higher) = reward(world.pois, town, plan, making.chain, daily, rng);
    let tale = Tale {
        household: household(town, making.building),
        member: making.member,
        text_seed: making.text_seed,
        reward: reward.clone(),
        higher,
        earlier: making.earlier,
        dark: making.dark,
        continues: making.continues,
        daily,
    };
    let telling = tell_plan(world, town, plan, tale);
    Quest {
        id: making.id,
        kind: making.kind,
        building: making.building,
        member: making.member,
        requirements: making.requirements,
        steps: plan.steps.clone(),
        reward,
        deeds: deeds(plan, making.dark),
        story: telling.story,
        ending: telling.ending,
    }
}

/// What finishing it does to alignment: rescuing someone, seeing a pet home,
/// or helping. Dark quests' deeds are left to the Quest system's PR 10.
fn deeds(plan: &Plan, dark: bool) -> Vec<Deed> {
    if dark {
        return Vec::new();
    }
    let escort = plan.steps.iter().any(|step| step.goal == Goal::Escort);
    vec![match (escort, plan.pet.is_some()) {
        (true, true) => Deed::Escorted,
        (true, false) => Deed::Rescued,
        _ => Deed::Helped,
    }]
}
