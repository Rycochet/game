//! A town's quests: every giver's slots, each a chain of one to three quests
//! on hooks near the town; then needs on smaller towns' feeders, and the
//! town's own feeder.

use std::collections::{BTreeMap, BTreeSet};

use super::area::{Candidate, candidates};
use super::feeder_quest::{add_feeder_needs, feeder_quest};
use super::slot::{Claims, slot_quests};
use super::world::{MOST_RADIUS, pois_within, radius};
use super::{Quest, QuestWorld, givers};
use crate::poi::Poi;
use crate::site::BuildingLook;

/// Every normal quest `town` offers, givers in the buildings' order, each
/// slot's chain in order, then its feeder. `buildings` are its site's
/// (`SiteLevel::buildings`' looks). Empty for anything but a town.
pub fn town_quests(world: &QuestWorld, town: &Poi, buildings: &[BuildingLook]) -> Vec<Quest> {
    if !town.kind.is_town() {
        return Vec::new();
    }
    let candidates = town_candidates(world, town, false);
    let mut claims = Claims {
        taken: BTreeSet::new(),
        uses: BTreeMap::new(),
    };
    let mut quests = Vec::new();
    for giver in givers(town, buildings) {
        for slot in 0..giver.slots {
            quests.extend(slot_quests(
                world,
                town,
                &giver,
                slot,
                &candidates,
                &mut claims,
            ));
        }
    }
    add_feeder_needs(world, town, &mut quests);
    quests.extend(feeder_quest(world, town, buildings));
    quests
}

/// What `town`'s quests (or dailies) may use.
pub(super) fn town_candidates(world: &QuestWorld, town: &Poi, daily: bool) -> Vec<Candidate> {
    let reach = radius(town.kind, daily) + MOST_RADIUS;
    let towns: Vec<Poi> = (pois_within(world.pois, town, reach).into_iter())
        .filter(|poi| poi.kind.is_town())
        .collect();
    candidates(world.pois, town, &towns, daily)
}
