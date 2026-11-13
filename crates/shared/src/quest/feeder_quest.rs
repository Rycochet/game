//! A town's own feeder quest (taking something to a larger neighbour), and
//! the needs its quests put on smaller neighbours' feeders.

use super::givers::quest_seed;
use super::make::{Making, make};
use super::plan::Plan;
use super::{
    Need, Quest, QuestId, QuestKind, QuestWorld, Requirement, Slot, Step, Target,
    feeder_destination, feeds_into, givers,
};
use crate::poi::Poi;
use crate::rng::Seed;
use crate::site::BuildingLook;
use crate::story::{Goal, ItemCategory};

/// What a feeder takes.
const SENT: [ItemCategory; 4] = [
    ItemCategory::Letter,
    ItemCategory::Goods,
    ItemCategory::Box,
    ItemCategory::Remedy,
];

/// `town`'s feeder quest, if a larger town is joined to it by road. Its giver
/// is one of the town's givers (else any building), member 0.
pub(super) fn feeder_quest(
    world: &QuestWorld,
    town: &Poi,
    buildings: &[BuildingLook],
) -> Option<Quest> {
    let destination = feeder_destination(world, town)?;
    let seed = quest_seed(town).derive("feeder quest");
    let mut rng = seed.derive("plan").rng();
    let givers: Vec<u16> = (givers(town, buildings).iter())
        .filter(|g| g.slots > 0)
        .map(|g| g.building)
        .collect();
    let choices: Vec<u16> = if givers.is_empty() {
        buildings.iter().map(|b| b.id).collect()
    } else {
        givers
    };
    let building = *choices.get(rng.range_u64(0, choices.len().max(1) as u64) as usize)?;
    let plan = Plan {
        steps: vec![Step {
            goal: Goal::Deliver,
            target: Target::Town {
                chunk: destination.chunk,
            },
            count: None,
            after: Vec::new(),
            alternatives: Vec::new(),
        }],
        item: Some(SENT[rng.range_u64(0, SENT.len() as u64) as usize]),
        destination: Some(destination),
        ..Plan::default()
    };
    let making = Making {
        id: QuestId {
            town: town.chunk,
            slot: Slot::Feeder,
        },
        kind: QuestKind::Normal,
        building,
        member: 0,
        requirements: Vec::new(),
        chain: 0,
        earlier: None,
        dark: false,
        continues: false,
        text_seed: seed.derive("text"),
    };
    Some(make(world, town, &plan, making, &mut rng))
}

/// For each smaller town whose feeder leads here, one first quest of a chain
/// (not a dark one) needs it done, each a different one, by a hash.
pub(super) fn add_feeder_needs(world: &QuestWorld, town: &Poi, quests: &mut [Quest]) {
    let seed = quest_seed(town).derive("feeder needs");
    for smaller in feeds_into(world, town) {
        let open: Vec<usize> = (0..quests.len())
            .filter(|&i| {
                let quest = &quests[i];
                matches!(quest.id.slot, Slot::Giver { chain: 0, .. })
                    && !quest.is_hidden_by_alignment()
                    && !(quest.requirements.iter()).any(|r| matches!(r.need, Need::Done(_)))
            })
            .collect();
        if open.is_empty() {
            return;
        }
        let hash = Seed::new(seed.at(&[smaller.chunk.x, smaller.chunk.y])).value();
        let quest = &mut quests[open[(hash % open.len() as u64) as usize]];
        quest
            .requirements
            .push(Requirement::shown(Need::Done(QuestId {
                town: smaller.chunk,
                slot: Slot::Feeder,
            })));
    }
}
