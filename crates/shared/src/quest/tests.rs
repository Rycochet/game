use std::collections::{BTreeMap, BTreeSet};

use super::test_world::{TestWorld, buildings};
use super::*;
use crate::lang::{Lang, Language};
use crate::poi::{ChunkPos, Poi};
use crate::story::RewardPart;

/// Chunks `0..SIDE` squared hold a few dozen towns of every size.
const SIDE: i64 = 24;

pub(super) fn every_town(world: &TestWorld) -> Vec<(Poi, Vec<Quest>)> {
    (world.towns(SIDE).into_iter())
        .map(|town| {
            let quests = town_quests(&world.view(), &town, &buildings(&town));
            (town, quests)
        })
        .collect()
}

#[test]
fn the_same_town_always_has_the_same_quests() {
    let world = TestWorld::new();
    let town = world.towns(SIDE)[0];
    let once = town_quests(&world.view(), &town, &buildings(&town));
    let again = town_quests(&world.view(), &town, &buildings(&town));
    assert!(!once.is_empty());
    assert_eq!(once, again);
}

#[test]
fn quests_keep_to_the_rules() {
    let world = TestWorld::new();
    let lang = Lang::new(Language::English);
    let towns = every_town(&world);
    assert!(towns.len() >= 10, "{}", towns.len());
    let mut claimed: BTreeMap<(ChunkPos, Hook), QuestId> = BTreeMap::new();
    for (town, quests) in &towns {
        assert!(!quests.is_empty(), "{town:?}");
        for quest in quests {
            assert_eq!(quest.id.town, town.chunk);
            assert!((1..=2).contains(&quest.steps.len()), "{quest:?}");
            for (i, step) in quest.steps.iter().enumerate() {
                assert!(step.after.iter().all(|&a| usize::from(a) < i), "{quest:?}");
                if let Target::Poi { chunk, hook } = step.target {
                    let poi = world.pois.in_chunk(chunk).unwrap();
                    assert!(Hook::of_poi(&poi).contains(&hook));
                    let reach = radius(town.kind, quest.kind == QuestKind::Daily);
                    assert!(super::world::within(&world.pois, town, &poi, reach));
                    if hook.is_exclusive() {
                        assert_eq!(owner(&world.view(), &poi, hook), Some(town.chunk));
                        let first = claimed.insert((chunk, hook), quest.id);
                        assert_eq!(first, None, "{chunk:?} {hook:?} used twice");
                    }
                }
            }
            assert!(matches!(quest.reward[0], RewardPart::Coins(c) if c > 0));
            assert!(!quest.deeds.is_empty() || quest.is_hidden_by_alignment());
            let story = &quest.story;
            assert_eq!(story.facts.asks.len(), quest.steps.len());
            story.title(&lang).unwrap();
            story.paragraphs_text(&lang).unwrap();
            for index in 0..quest.steps.len() {
                story.step_text(&lang, index, 0).unwrap();
            }
            story.ending_text(&lang, &quest.ending).unwrap();
        }
    }
}

#[test]
fn chains_need_the_quest_before_and_keep_its_story() {
    let world = TestWorld::new();
    let mut chained = 0;
    for (_, quests) in every_town(&world) {
        for quest in &quests {
            let Some(earlier_id) = quest.id.earlier() else {
                continue;
            };
            chained += 1;
            let earlier = quests.iter().find(|q| q.id == earlier_id).unwrap();
            assert!(
                quest
                    .requirements
                    .contains(&Requirement::shown(Need::Done(earlier_id)))
            );
            assert_eq!(quest.story.reason, earlier.story.reason);
            assert_eq!(
                quest.story.cast.giver().person,
                earlier.story.cast.giver().person
            );
            assert_eq!(quest.member, earlier.member);
            assert_eq!(
                quest.is_hidden_by_alignment(),
                earlier.is_hidden_by_alignment()
            );
        }
    }
    assert!(chained > 0);
}

#[test]
fn chains_across_towns_only_need_smaller_neighbours() {
    let world = TestWorld::new();
    let (mut feeders, mut needs) = (0, 0);
    for (town, quests) in every_town(&world) {
        for quest in &quests {
            if quest.id.slot == Slot::Feeder {
                feeders += 1;
                let Target::Town { chunk } = quest.steps[0].target else {
                    panic!("{quest:?}");
                };
                let larger = world.pois.in_chunk(chunk).unwrap();
                assert!(is_larger(&larger, &town));
                assert!(quest.story.facts.destination.is_some());
            }
            for requirement in &quest.requirements {
                let Need::Done(QuestId { town: other, slot }) = requirement.need else {
                    continue;
                };
                if other == town.chunk {
                    continue;
                }
                needs += 1;
                assert_eq!(slot, Slot::Feeder);
                let smaller = world.pois.in_chunk(other).unwrap();
                assert!(is_larger(&town, &smaller));
                let to = feeder_destination(&world.view(), &smaller).unwrap();
                assert_eq!(to.chunk, town.chunk);
            }
        }
    }
    assert!(feeders > 0 && needs > 0, "{feeders} feeders, {needs} needs");
}

#[test]
fn dailies_are_short_shared_and_the_same_all_day() {
    let world = TestWorld::new();
    let view = world.view();
    let mut seen = 0;
    for town in world.towns(SIDE) {
        let buildings = buildings(&town);
        for building in daily_givers(&town, &buildings) {
            let day = utc_day(1_800_000_000);
            let quest = daily_quest(&view, &town, &buildings, building, day).unwrap();
            let again = daily_quest(&view, &town, &buildings, building, day).unwrap();
            assert_eq!(quest, again);
            assert_eq!(quest.kind, QuestKind::Daily);
            assert_eq!(quest.story.paragraphs.len(), 1);
            assert!(quest.requirements.is_empty());
            for step in &quest.steps {
                if let Target::Poi { hook, .. } = step.target {
                    assert!(!hook.is_exclusive());
                }
            }
            seen += 1;
        }
    }
    assert!(seen > 0);
}

#[test]
fn non_givers_and_non_towns_have_no_quests() {
    let world = TestWorld::new();
    let town = world.towns(SIDE)[0];
    let buildings = buildings(&town);
    let daily = daily_givers(&town, &buildings);
    let other = (buildings.iter()).find(|b| !daily.contains(&b.id)).unwrap();
    assert!(daily_quest(&world.view(), &town, &buildings, other.id, 0).is_none());
    let not_a_town = Poi {
        kind: crate::poi::PoiKind::RockCave,
        ..town
    };
    assert!(town_quests(&world.view(), &not_a_town, &buildings).is_empty());
}

#[test]
fn steps_open_as_those_before_them_are_done() {
    let world = TestWorld::new();
    let quest = (every_town(&world).into_iter())
        .flat_map(|(_, quests)| quests)
        .find(|q| q.steps.len() == 2 && q.steps[1].after == [0])
        .expect("a quest in series");
    assert_eq!(quest.open_steps(&[]), [0]);
    assert_eq!(quest.open_steps(&[true]), [1]);
    assert!(!quest.ready_to_turn_in(&[true]));
    assert!(quest.ready_to_turn_in(&[true, true]));
}

#[test]
fn days_are_utc() {
    assert_eq!(utc_day(0), 0);
    assert_eq!(utc_day(SECONDS_PER_DAY - 1), 0);
    assert_eq!(utc_day(SECONDS_PER_DAY), 1);
    assert_eq!(utc_day(-1), -1);
}

#[test]
fn larger_is_by_size_then_chunk() {
    use crate::poi::PoiKind::*;
    let town = |kind, x| Poi {
        kind,
        chunk: ChunkPos { x, y: 0 },
        x: x * 64,
        y: 0,
        seed: crate::rng::Seed::new(1),
        desert: false,
    };
    assert!(is_larger(&town(City, 0), &town(Hamlet, 5)));
    assert!(is_larger(&town(Village, 5), &town(Village, 4)));
    assert!(!is_larger(&town(Village, 4), &town(Village, 4)));
}

#[test]
fn an_area_has_every_kind_of_quest() {
    let world = TestWorld::new();
    let quests: Vec<Quest> = (every_town(&world).into_iter())
        .flat_map(|(_, quests)| quests)
        .collect();
    let goals: BTreeSet<_> = (quests.iter())
        .flat_map(|q| q.steps.iter().map(|s| s.goal))
        .collect();
    assert_eq!(goals.len(), crate::story::Goal::ALL.len(), "{goals:?}");
    let any = |test: &dyn Fn(&Quest) -> bool| quests.iter().any(test);
    assert!(any(&|q| q.is_hidden_by_alignment()), "a dark chain");
    assert!(
        any(&|q| q.requirements.iter().any(|r| r.hidden)),
        "a hidden need"
    );
    assert!(
        any(&|q| q.steps.iter().any(|s| !s.alternatives.is_empty())),
        "another way"
    );
    assert!(
        any(&|q| q.steps.len() == 2 && q.steps[1].after.is_empty()),
        "in parallel"
    );
    assert!(any(&|q| q.reward.len() > 1), "a higher reward");
    let collects = |q: &Quest| {
        (q.steps.iter()).any(|s| {
            matches!(
                s.target,
                Target::Poi {
                    hook: Hook::Collect(_),
                    ..
                }
            )
        })
    };
    assert!(any(&collects), "something to collect");
    for quest in quests.iter().filter(|q| collects(q)) {
        let remains = quest.story.facts.remains.expect("what to bring back");
        let natural = super::remains::natural_parts(remains.kind);
        assert!(natural.iter().any(|(part, _)| *part == remains.part));
    }
}

#[test]
fn parts_to_collect_are_natural_to_the_creature() {
    use crate::creature::CreatureKind::{Chicken, Human, Minotaur, Wolf};
    use crate::story::BodyPart::{Feather, Fingernail, Hoof, Horn, Tail, Toenail};
    let has = |kind, part| (super::remains::natural_parts(kind).iter()).any(|(p, _)| *p == part);
    assert!(has(Human, Fingernail) && has(Human, Toenail));
    assert!(!has(Wolf, Fingernail) && !has(Wolf, Toenail) && has(Wolf, Tail));
    assert!(has(Minotaur, Horn) && has(Minotaur, Hoof));
    assert!(has(Chicken, Feather) && !has(Human, Feather));
}

#[test]
fn a_poi_boss_has_one_tier_above_common() {
    let world = TestWorld::new();
    let mut tiers = BTreeSet::new();
    for (_, quests) in every_town(&world) {
        for quest in quests {
            for step in &quest.steps {
                if let Target::Poi {
                    chunk,
                    hook: Hook::Boss,
                } = step.target
                {
                    let poi = world.pois.in_chunk(chunk).expect("its poi");
                    let tier = boss_tier(&poi);
                    assert_eq!(tier, boss_tier(&poi), "the same each time");
                    assert!(tier > crate::item::Quality::Common, "{tier:?}");
                    tiers.insert(tier);
                }
            }
        }
    }
    assert!(tiers.len() > 1, "more than one tier: {tiers:?}");
}
