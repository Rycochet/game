//! Chains between a town's givers, quests waiting for a moon, and dark
//! quests' deeds.

use super::test_world::TestWorld;
use super::tests::every_town;
use super::*;
use crate::alignment::Deed;

#[test]
fn givers_need_each_other_one_deep() {
    let world = TestWorld::new();
    let mut found = 0;
    for (town, quests) in every_town(&world) {
        let by_id = |id: QuestId| quests.iter().find(|quest| quest.id == id);
        for quest in &quests {
            for requirement in &quest.requirements {
                let Need::Done(id) = requirement.need else {
                    continue;
                };
                if id.town != town.chunk || quest.id.earlier() == Some(id) {
                    continue;
                }
                // Another giver's first quest, which needs nothing done.
                found += 1;
                assert!(
                    matches!(quest.id.slot, Slot::Giver { chain: 0, .. }),
                    "{quest:?}"
                );
                let other = by_id(id).expect("needs a quest of its town");
                assert_ne!(other.building, quest.building);
                assert!(matches!(other.id.slot, Slot::Giver { chain: 0, .. }));
                assert!(!(other.requirements.iter()).any(|r| matches!(r.need, Need::Done(_))));
                assert!(!quest.is_hidden_by_alignment());
            }
        }
    }
    assert!(found > 0, "no givers need each other");
}

#[test]
fn moons_are_waited_for_out_of_sight() {
    let world = TestWorld::new();
    let mut found = 0;
    for (town, quests) in every_town(&world) {
        for quest in &quests {
            for requirement in &quest.requirements {
                if let Need::Moon { moon } = requirement.need {
                    found += 1;
                    assert!(requirement.hidden);
                    assert!(moon < world.view().moons);
                    assert!(super::world::size(town.kind).unwrap_or(0) > 0);
                }
            }
        }
    }
    assert!(found > 0, "no quest waits for a moon");
}

#[test]
fn worlds_without_moons_wait_for_none() {
    let world = TestWorld::new();
    let view = QuestWorld {
        moons: 0,
        ..world.view()
    };
    for town in world.towns(12) {
        let quests = town_quests(&view, &town, &super::test_world::buildings(&town));
        assert!(
            (quests.iter().flat_map(|quest| &quest.requirements))
                .all(|r| !matches!(r.need, Need::Moon { .. }))
        );
    }
}

#[test]
fn dark_quests_weigh_against_alignment() {
    let world = TestWorld::new();
    let mut dark = 0;
    for (_, quests) in every_town(&world) {
        for quest in &quests {
            if quest.is_hidden_by_alignment() {
                dark += 1;
                assert_eq!(quest.deeds, vec![Deed::DarkQuest]);
                assert!(Deed::DarkQuest.shift() < 0);
            } else {
                assert!(!quest.deeds.contains(&Deed::DarkQuest));
            }
        }
    }
    assert!(dark > 0, "no dark quests");
}
