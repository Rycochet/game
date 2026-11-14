//! Lone quest bosses: on their site's deepest level from the start, the sign
//! over them for each character, a kill without the quest remembered and
//! offered as already dealt with, and coming back for whoever still wants it.

use shared::poi::{CHUNK_SIZE, ChunkPos, Poi};
use shared::protocol::{Ahead, BossSign, Hint, OfferState, QuestNews, ServerMessage};
use shared::quest::{Hook, Quest, Target, boss_of};

use super::bosses::{BossQuest, Regard, boss_quest, regard};
use super::tests::NOW;
use super::{accept::accept, boss_kill::slew_boss, cache, offers};
use crate::connections::tests::TestClient;
use crate::sites::test_enter_poi;
use crate::storage::{Progress, SiteId};

/// A quest near the start with a boss to slay, and its boss's poi.
fn a_boss_quest(test: &mut TestClient) -> (Quest, Poi) {
    let (x, y) = test.client.world_position();
    let reach = 16 * CHUNK_SIZE;
    let mut towns: Vec<ChunkPos> = (test.world.pois)
        .in_area((x - reach, y - reach), (x + reach, y + reach))
        .into_iter()
        .filter(|poi| poi.kind.is_town())
        .map(|poi| poi.chunk)
        .collect();
    towns.sort_unstable();
    for chunk in towns {
        let town = cache::town(&mut test.world, chunk).expect("a town");
        for quest in town.quests.iter() {
            let boss = quest.steps.iter().find_map(|step| match step.target {
                Target::Poi {
                    chunk,
                    hook: Hook::Boss,
                } => Some(chunk),
                _ => None,
            });
            if let Some(at) = boss {
                let poi = test.world.pois.in_chunk(at).expect("its poi");
                return (quest.clone(), poi);
            }
        }
    }
    panic!("no quest near the start with a boss");
}

fn found(test: &mut TestClient, poi: &Poi) -> BossQuest {
    boss_quest(&mut test.world, poi.chunk).expect("the poi's boss has its quest")
}

/// Into the poi's deepest level; its number.
fn to_the_bottom(test: &mut TestClient, poi: Poi) -> u32 {
    assert!(test_enter_poi(&mut test.client, &mut test.world, poi, 0));
    let levels = test.client.visit.as_ref().unwrap().site().unwrap().levels();
    assert!(test_enter_poi(
        &mut test.client,
        &mut test.world,
        poi,
        levels - 1
    ));
    levels - 1
}

fn boss_on(test: &TestClient, place: (SiteId, u32)) -> Option<shared::id::Id> {
    let population = test.world.populations.0.get(&place)?;
    let boss = population.mobs.iter().find(|mob| mob.is_boss())?;
    Some(boss.id)
}

fn taken(quest: &Quest) -> Progress {
    Progress {
        at: NOW,
        counts: vec![0; quest.steps.len()],
        ready: false,
        escort: None,
    }
}

#[test]
fn a_poi_boss_belongs_to_the_one_quest_that_asks_for_it() {
    let mut test = TestClient::new();
    let (quest, poi) = a_boss_quest(&mut test);
    let boss = found(&mut test, &poi);
    assert_eq!(boss.id, quest.id);
    let wanted = Target::Poi {
        chunk: poi.chunk,
        hook: Hook::Boss,
    };
    assert_eq!(quest.steps[boss.step].target, wanted);
}

#[test]
fn the_boss_is_on_its_sites_deepest_level_only() {
    let mut test = TestClient::new();
    let (_, poi) = a_boss_quest(&mut test);
    let last = to_the_bottom(&mut test, poi);
    let place = (SiteId::Poi(poi.chunk), last);
    let population = test.world.populations.0.get(&place).unwrap();
    let boss = population.mobs.iter().find(|mob| mob.is_boss());
    assert_eq!(boss.map(|mob| mob.kind), Some(boss_of(&poi).kind));
    if last > 0 {
        assert!(test_enter_poi(&mut test.client, &mut test.world, poi, 0));
        assert!(boss_on(&test, (SiteId::Poi(poi.chunk), 0)).is_none());
    }
}

#[test]
fn the_sign_over_a_boss_is_red_for_strangers_and_the_quests_for_questers() {
    let mut test = TestClient::new();
    let (quest, poi) = a_boss_quest(&mut test);
    let boss = found(&mut test, &poi);
    let book = test.client.character.quests.clone();
    assert_eq!(regard(&mut test.world, &book, boss), Regard::Stranger);

    let last = to_the_bottom(&mut test, poi);
    let id = boss_on(&test, (SiteId::Poi(poi.chunk), last)).unwrap();
    test.send(&[]);
    test.client.seen_entities.insert(id, (0, 0));
    super::boss_signs::tell_bosses(&mut test.client, &mut test.world);
    let told = test
        .send(&[])
        .into_iter()
        .find_map(|message| match message {
            ServerMessage::Quest(QuestNews::Bosses(marks)) => Some(marks),
            _ => None,
        });
    let marks = told.expect("told of the boss");
    assert_eq!(marks.len(), 1);
    assert_eq!((marks[0].id, marks[0].sign), (id, BossSign::Unknown));

    let book = &mut test.client.character.quests;
    book.taken.insert(quest.id, taken(&quest));
    let book = book.clone();
    assert_eq!(regard(&mut test.world, &book, boss), Regard::OnStep);
    assert_eq!(
        Regard::OnStep.sign(quest.kind),
        Some(BossSign::Quest(quest.kind))
    );
}

#[test]
fn killed_without_the_quest_it_is_remembered_and_offered_as_dealt_with() {
    let mut test = TestClient::new();
    let (quest, poi) = a_boss_quest(&mut test);
    let boss = found(&mut test, &poi);
    assert!(slew_boss(&mut test.client, &mut test.world, poi.chunk, NOW));
    let book = test.client.character.quests.clone();
    assert!(book.slain.contains(&quest.id));
    let town = test.client.character.pois.get(&quest.id.town);
    assert!(town.is_some_and(|known| known.known && known.named));
    assert_eq!(regard(&mut test.world, &book, boss), Regard::Dealt);

    let (town, building) = (quest.id.town, quest.building);
    let offered = offers::giver_offers(&test.client, &mut test.world, town, building, NOW);
    let state = offered.iter().find(|offer| offer.quest.id == quest.id);
    match state.map(|offer| &offer.state) {
        Some(OfferState::Available { ahead, .. }) => assert_eq!(*ahead, Some(Ahead::Slain)),
        Some(OfferState::Locked(hints)) => assert!(hints.contains(&Hint::Ahead(Ahead::Slain))),
        // A later quest of a chain: offered once the one before is done.
        None => return,
        other => panic!("not offered as dealt with: {other:?}"),
    }
    if quest.requirements.is_empty() {
        assert!(accept(
            &mut test.client,
            &mut test.world,
            quest.id,
            NOW,
            false
        ));
        let book = &test.client.character.quests;
        assert!(!book.slain.contains(&quest.id), "kept only until taken");
        assert_eq!(book.taken[&quest.id].counts[boss.step], 1);
    }
}

#[test]
fn a_quester_on_the_step_who_kills_it_has_the_step_done() {
    let mut test = TestClient::new();
    let (quest, poi) = a_boss_quest(&mut test);
    let boss = found(&mut test, &poi);
    let book = &mut test.client.character.quests;
    book.taken.insert(quest.id, taken(&quest));
    assert!(slew_boss(&mut test.client, &mut test.world, poi.chunk, NOW));
    let book = &test.client.character.quests;
    assert_eq!(book.taken[&quest.id].counts[boss.step], 1);
    assert!(book.slain.is_empty());
}

#[test]
fn a_dead_boss_comes_back_for_whoever_still_wants_it() {
    let mut test = TestClient::new();
    let (_, poi) = a_boss_quest(&mut test);
    let last = to_the_bottom(&mut test, poi);
    let place = (SiteId::Poi(poi.chunk), last);
    // It dies, and is down.
    let population = test.world.populations.0.get_mut(&place).unwrap();
    population.mobs.retain(|mob| !mob.is_boss());
    let saved = test.world.site_mut(place.0).levels.entry(last).or_default();
    saved.dead.insert(crate::entities::BOSS_NUMBER);

    // Someone who killed it, with no quest: it stays down.
    let boss = found(&mut test, &poi);
    test.client.character.quests.slain.insert(boss.id);
    super::arrivals::quest_mobs(&test.client, &mut test.world);
    assert!(boss_on(&test, place).is_none());

    // Someone who hasn't: it comes back.
    test.client.character.quests.slain.clear();
    super::arrivals::quest_mobs(&test.client, &mut test.world);
    assert!(boss_on(&test, place).is_some());
}
