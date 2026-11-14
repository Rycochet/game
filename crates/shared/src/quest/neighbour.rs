//! Chains between a town's givers: some givers' first quests need another
//! giver's first quest done ("I heard you helped Mara Tull"), so word of a
//! character gets round a town (owner).

use std::collections::BTreeSet;

use super::givers::quest_seed;
use super::{Need, Quest, Requirement, Slot};
use crate::poi::Poi;
use crate::rng::Seed;

/// Percent of givers' first quests that need another giver's first quest
/// done. *To balance.*
pub(super) const NEIGHBOUR_PERCENT: u64 = 25;

/// Gives some first quests of chains a need for another giver's first
/// quest. Only quests needing nothing done are used, either way, so needs go
/// one deep and never round in a circle.
pub(super) fn add_neighbour_needs(town: &Poi, quests: &mut [Quest]) {
    let seed = quest_seed(town).derive("neighbour needs");
    let free = |quest: &Quest| {
        matches!(quest.id.slot, Slot::Giver { chain: 0, .. })
            && !quest.is_hidden_by_alignment()
            && !(quest.requirements.iter()).any(|r| matches!(r.need, Need::Done(_)))
    };
    let free: Vec<usize> = (0..quests.len()).filter(|&i| free(&quests[i])).collect();
    let mut needing = BTreeSet::new();
    let mut needed = BTreeSet::new();
    for &i in &free {
        if needed.contains(&i) {
            continue;
        }
        let Slot::Giver { building, slot, .. } = quests[i].id.slot else {
            continue;
        };
        let mut rng = Seed::new(seed.at(&[i64::from(building), i64::from(slot)])).rng();
        if rng.range_u64(0, 100) >= NEIGHBOUR_PERCENT {
            continue;
        }
        let others: Vec<usize> = (free.iter().copied())
            .filter(|&j| quests[j].building != building && !needing.contains(&j))
            .collect();
        if others.is_empty() {
            continue;
        }
        let other = others[rng.range_u64(0, others.len() as u64) as usize];
        let id = quests[other].id;
        quests[i]
            .requirements
            .push(Requirement::shown(Need::Done(id)));
        needing.insert(i);
        needed.insert(other);
    }
}
