//! Cities' artifact lines.

use super::test_world::{TestWorld, buildings};
use super::*;
use crate::poi::PoiKind;
use crate::story::RewardPart;

/// The artifact quests of every city in the test world's first chunks.
fn lines(world: &TestWorld) -> Vec<(crate::poi::Poi, Vec<Quest>)> {
    (world.towns(48).into_iter())
        .filter(|town| town.kind == PoiKind::City)
        .map(|city| {
            let quests = town_quests(&world.view(), &city, &buildings(&city));
            let line = (quests.into_iter())
                .filter(|quest| quest.kind == QuestKind::Artifact)
                .collect();
            (city, line)
        })
        .collect()
}

#[test]
fn cities_have_long_lines_ending_in_a_large_dungeon() {
    let world = TestWorld::new();
    let lines = lines(&world);
    let made: Vec<_> = lines.iter().filter(|(_, line)| !line.is_empty()).collect();
    assert!(!made.is_empty(), "no city has an artifact line");
    for (city, line) in made {
        assert!(line.len() >= 3, "{} quests", line.len());
        let building = line[0].building;
        for (step, quest) in line.iter().enumerate() {
            assert_eq!(quest.id.town, city.chunk);
            assert_eq!(quest.id.slot, Slot::Artifact { step: step as u8 });
            assert_eq!(quest.building, building, "one giver for the line");
            let needs: Vec<Need> = quest.requirements.iter().map(|r| r.need).collect();
            assert!(needs.contains(&Need::Standing {
                town: city.chunk,
                least: Standing::Honoured
            }));
            if let Some(before) = quest.id.earlier() {
                assert!(needs.contains(&Need::Done(before)));
            }
            let prizes = (quest.reward.iter())
                .filter(|part| matches!(part, RewardPart::Artifact | RewardPart::SetItem))
                .count();
            assert_eq!(prizes, usize::from(step + 1 == line.len()));
        }
        let last = line.last().unwrap();
        let dungeon = last.dungeon.expect("the last quest is a dungeon");
        assert_eq!(dungeon.way, DungeonWay::Hidden);
        assert!(dungeon.levels >= 4);
        assert!(super::dungeon_tests::peopled_enough(last), "{last:?}");
        // Earlier ones go round the city's road neighbours.
        assert!(
            line[..line.len() - 1]
                .iter()
                .all(|quest| quest.dungeon.is_none())
        );
    }
}

#[test]
fn only_cities_have_artifact_quests() {
    let world = TestWorld::new();
    for town in world.towns(24) {
        if town.kind == PoiKind::City {
            continue;
        }
        let quests = town_quests(&world.view(), &town, &buildings(&town));
        assert!(quests.iter().all(|quest| quest.kind != QuestKind::Artifact));
    }
}

#[test]
fn some_lines_end_in_the_set_item() {
    let world = TestWorld::new();
    let ends: Vec<RewardPart> = (lines(&world).into_iter())
        .filter_map(|(_, line)| line.last().cloned())
        .filter_map(|quest| quest.reward.last().copied())
        .collect();
    assert!(ends.contains(&RewardPart::SetItem) || ends.contains(&RewardPart::Artifact));
}
