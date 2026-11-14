//! One giver's slot: a chain of one to three quests, each needing the one
//! before and more standing, keeping the first one's cast and reason.

use std::collections::{BTreeMap, BTreeSet};

use super::area::Candidate;
use super::givers::quest_seed;
use super::hidden::{HIDDEN_PERCENT, plan_hidden};
use super::make::{Making, make};
use super::plan::Plan;
use super::template::plan;
use super::world::size;
use super::{
    Giver, Hook, Need, Quest, QuestId, QuestKind, QuestWorld, Requirement, Slot, Standing,
};
use crate::alignment::Band;
use crate::poi::{ChunkPos, Poi};
use crate::rng::{Rng, Seed};
use crate::skill::SkillId;
use crate::story::Goal;

/// What a town's quests have used so far: exclusive hooks taken, and how
/// often each shared one is used (each use halves its chances).
pub(super) struct Claims {
    pub taken: BTreeSet<(ChunkPos, Hook)>,
    pub uses: BTreeMap<(ChunkPos, Hook), u64>,
}

/// Percent of slots (outside hamlets) whose chain only characters with a
/// bad name are offered, and percent of gathering quests with a hidden
/// herbalism need. *To balance.*
const DARK_PERCENT: u64 = 8;
const HERBALISM_PERCENT: u64 = 30;

/// The quests of `giver`'s `slot`, first to last.
pub(super) fn slot_quests(
    world: &QuestWorld,
    town: &Poi,
    giver: &Giver,
    slot: u8,
    candidates: &[Candidate],
    claims: &mut Claims,
) -> Vec<Quest> {
    let building = giver.building;
    let seed =
        Seed::new((quest_seed(town).derive("slot")).at(&[i64::from(building), i64::from(slot)]));
    let mut rng = seed.derive("plan").rng();
    let size = size(town.kind).unwrap_or(0);
    let longest = if size == 0 { 2 } else { 3 };
    let length = rng.range_u64(1, longest + 1);
    let dark = size > 0 && rng.range_u64(0, 100) < DARK_PERCENT;
    let mut plans = Vec::new();
    for _ in 0..length {
        // Sometimes a cellar of the giver's own, or in towns and cities a
        // hidden way down nearby; otherwise a hook nearby.
        let roll = rng.range_u64(0, 100);
        let cellars = super::dungeon::NORMAL_PERCENT;
        let hidden = (size >= 2 && roll >= cellars && roll < cellars + HIDDEN_PERCENT)
            .then(|| plan_hidden(world, town, &mut rng))
            .flatten();
        let plan = if roll < cellars {
            super::dungeon::plan_cellar(town, false, &mut rng)
        } else if let Some(hidden) = hidden {
            hidden
        } else {
            let Some(main) = choose(candidates, claims, &mut rng) else {
                break;
            };
            plan(town, &main, candidates, &mut rng)
        };
        for &claim in &plan.claims {
            claims.taken.insert(claim);
        }
        for step in &plan.steps {
            let key = (step.target.chunk(), hook_of(step));
            *claims.uses.entry(key).or_default() += 1;
        }
        plans.push(plan);
    }
    let mut quests: Vec<Quest> = Vec::new();
    for (chain, plan) in plans.iter().enumerate() {
        let chain = chain as u8;
        let id = QuestId {
            town: town.chunk,
            slot: Slot::Giver {
                building,
                slot,
                chain,
            },
        };
        let making = Making {
            id,
            kind: QuestKind::Normal,
            building,
            member: u32::from(slot),
            requirements: requirements(town, id, plan, dark, &mut rng),
            chain,
            earlier: quests.last().map(|q| &q.story),
            dark,
            continues: usize::from(chain) + 1 < plans.len(),
            text_seed: Seed::new(seed.derive("text").at(&[i64::from(chain)])),
        };
        let quest = make(world, town, plan, making, &mut rng);
        quests.push(quest);
    }
    quests
}

/// A hook to build a quest on: by weight, less for shared ones already used,
/// never an exclusive one taken.
fn choose(candidates: &[Candidate], claims: &Claims, rng: &mut Rng) -> Option<Candidate> {
    let weights: Vec<(usize, u64)> = (candidates.iter().enumerate())
        .filter(|(_, c)| !claims.taken.contains(&(c.poi.chunk, c.hook)))
        .map(|(i, c)| {
            let used = claims
                .uses
                .get(&(c.poi.chunk, c.hook))
                .copied()
                .unwrap_or(0);
            (i, ((c.weight * 4) >> used.min(8)).max(1))
        })
        .collect();
    (!weights.is_empty()).then(|| candidates[rng.pick_weighted(&weights)])
}

fn hook_of(step: &super::Step) -> Hook {
    match step.target {
        super::Target::Poi { hook, .. } | super::Target::Dungeon { hook, .. } => hook,
        _ => Hook::Gather,
    }
}

/// What a chain's quest needs: the one before and more standing for later
/// ones, a bad name for dark chains, and sometimes a hidden skill.
fn requirements(
    town: &Poi,
    id: QuestId,
    plan: &Plan,
    dark: bool,
    rng: &mut Rng,
) -> Vec<Requirement> {
    let mut needs = Vec::new();
    if let Some(earlier) = id.earlier() {
        needs.push(Requirement::shown(Need::Done(earlier)));
        let chain = if let Slot::Giver { chain, .. } = id.slot {
            chain
        } else {
            0
        };
        needs.push(Requirement::shown(Need::Standing {
            town: town.chunk,
            least: Standing::ALL[usize::from(chain).min(Standing::ALL.len() - 1)],
        }));
    }
    if dark {
        needs.push(Requirement::shown(Need::Alignment(Band::Evil)));
    }
    let gathers = plan.steps.iter().any(|step| step.goal == Goal::Gather);
    if gathers
        && rng.range_u64(0, 100) < HERBALISM_PERCENT
        && let Some(skill) = SkillId::find("gathering/herbalism")
    {
        needs.push(Requirement::hidden(Need::Skill { skill, least: 1 }));
    }
    needs
}
