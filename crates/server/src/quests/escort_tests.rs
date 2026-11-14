//! Escorts: met on their poi's deepest level, following onto the world map
//! (seen there by the quester alone, at the slower's pace), seen home in the
//! giver's town, and failing the quest if killed.

use shared::movement::Gait;
use shared::poi::{CHUNK_SIZE, ChunkPos, Poi};
use shared::quest::{Hook, Quest, Target};

use super::escortee::escortee_id;
use super::tests::NOW;
use super::{cache, escort, follow};
use crate::connections::tests::TestClient;
use crate::sites::test_enter_poi;
use crate::storage::{EscortAt, Progress};

/// Quests near the start whose first step is an escort, each with the step
/// and the poi the escortee waits at.
fn escort_quests(test: &mut TestClient) -> Vec<(Quest, usize, Poi)> {
    let mut all = Vec::new();
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
            let escort = quest.steps.iter().position(|step| {
                matches!(
                    step.target,
                    Target::Poi {
                        hook: Hook::Escortee,
                        ..
                    }
                ) && step.after.is_empty()
            });
            if let Some(step) = escort {
                let at = quest.steps[step].target.chunk();
                let poi = test.world.pois.in_chunk(at).expect("its poi");
                all.push((quest.clone(), step, poi));
            }
        }
    }
    assert!(!all.is_empty(), "no quest near the start with an escort");
    all
}

/// Takes the first such quest and goes down to its escortee's level.
fn taken_and_there(test: &mut TestClient) -> (Quest, usize, Poi) {
    let first = escort_quests(test).remove(0);
    take_and_go(test, first)
}

fn take_and_go(
    test: &mut TestClient,
    (quest, step, poi): (Quest, usize, Poi),
) -> (Quest, usize, Poi) {
    let progress = Progress {
        at: NOW,
        counts: vec![0; quest.steps.len()],
        ready: false,
        escort: None,
    };
    test.client
        .character
        .quests
        .taken
        .insert(quest.id, progress);
    assert!(test_enter_poi(&mut test.client, &mut test.world, poi, 0));
    let levels = test.client.visit.as_ref().unwrap().site().unwrap().levels();
    if levels > 1 {
        assert!(test_enter_poi(
            &mut test.client,
            &mut test.world,
            poi,
            levels - 1
        ));
    }
    tick(test);
    (quest, step, poi)
}

fn tick(test: &mut TestClient) {
    follow(std::slice::from_mut(&mut test.client), &mut test.world);
}

fn state(test: &TestClient, quest: &Quest) -> Option<EscortAt> {
    test.client
        .character
        .quests
        .taken
        .get(&quest.id)
        .and_then(|p| p.escort)
}

/// Meets them where they wait: they follow from now on.
fn meet(test: &mut TestClient, quest: &Quest) {
    let mob = escortee_id(&test.client, quest.id);
    let (_, spot) = test.world.escortee_at(mob).expect("they wait here");
    let visit = test.client.visit.as_mut().unwrap();
    visit.move_for_test(spot.cell());
    tick(test);
    assert_eq!(state(test, quest), Some(EscortAt::With));
}

#[test]
fn they_wait_on_the_deepest_level_and_follow_once_met() {
    let mut test = TestClient::new();
    let (quest, _, _) = taken_and_there(&mut test);
    let mob = escortee_id(&test.client, quest.id);
    let (place, spot) = test.world.escortee_at(mob).expect("they wait here");
    assert_eq!(
        Some(place),
        test.client.visit.as_ref().map(|v| v.population())
    );
    assert!(matches!(
        state(&test, &quest),
        Some(EscortAt::Waiting { .. })
    ));
    // Still there, and no second one, however long the player stays.
    tick(&mut test);
    assert_eq!(test.world.escortee_at(mob).map(|at| at.1), Some(spot));
    meet(&mut test, &quest);
}

#[test]
fn on_the_world_map_only_the_quester_sees_them_and_they_set_the_pace() {
    let mut test = TestClient::new();
    let (quest, _, _) = taken_and_there(&mut test);
    meet(&mut test, &quest);
    assert_eq!(escort::world_gait(&test.client), Gait::UNSKILLED);
    assert!(crate::sites::put_out(&mut test.client, &mut test.world));
    tick(&mut test);
    // Off the level, with the player.
    let mob = escortee_id(&test.client, quest.id);
    assert!(test.world.escortee_at(mob).is_none());
    assert_eq!(state(&test, &quest), Some(EscortAt::With));
    let (look, _) = super::escort_on_world_map(&mut test.client, &mut test.world)
        .expect("drawn for the quester");
    assert_eq!(look.id, mob);
    assert_eq!((look.x, look.y), test.client.world_position());
}

#[test]
fn brought_into_the_givers_town_they_are_home() {
    let mut test = TestClient::new();
    let (quest, step, _) = taken_and_there(&mut test);
    meet(&mut test, &quest);
    let town = test.world.pois.in_chunk(quest.id.town).expect("the town");
    assert!(test_enter_poi(&mut test.client, &mut test.world, town, 0));
    tick(&mut test);
    let progress = &test.client.character.quests.taken[&quest.id];
    assert_eq!(progress.counts[step], 1, "the escort step is done");
    assert_eq!(progress.escort, None);
    let mob = escortee_id(&test.client, quest.id);
    assert!(test.world.escortee_at(mob).is_none(), "gone home");
}

#[test]
fn killed_the_quest_fails() {
    let mut test = TestClient::new();
    let (quest, _, _) = taken_and_there(&mut test);
    meet(&mut test, &quest);
    let quester = test.client.entity_id();
    let clients = std::slice::from_mut(&mut test.client);
    escort::escortee_killed(clients, &mut test.world, (quester, quest.id));
    let book = &test.client.character.quests;
    assert!(!book.taken.contains_key(&quest.id));
    assert!(book.ended[&quest.id].failed);
}

#[test]
fn left_far_behind_they_wait_and_near_they_come_along() {
    let mut test = TestClient::new();
    // One waiting on a level wide enough to be left behind on.
    let wide = (escort_quests(&mut test).into_iter()).find(|(_, _, poi)| {
        let mut test = TestClient::new();
        assert!(test_enter_poi(&mut test.client, &mut test.world, *poi, 0));
        let site = test.client.visit.as_ref().unwrap().site().unwrap().levels();
        assert!(test_enter_poi(
            &mut test.client,
            &mut test.world,
            *poi,
            site - 1
        ));
        let (_, _, width, height) = test.client.visit.as_ref().unwrap().cells().bounds();
        width.min(height) > 30
    });
    let (quest, _, poi) = take_and_go(&mut test, wide.expect("a wide level near the start"));
    meet(&mut test, &quest);
    let mob = escortee_id(&test.client, quest.id);
    let (place, spot) = test.world.escortee_at(mob).unwrap();
    let visit = test.client.visit.as_mut().unwrap();
    let (min_x, min_y, width, height) = visit.cells().bounds();
    let far = (min_y..min_y + height)
        .flat_map(|y| (min_x..min_x + width).map(move |x| (x, y)))
        .find(|&(x, y)| {
            let (cx, cy) = spot.cell();
            visit.walkable_for_test((x, y)) && (x - cx).abs().max((y - cy).abs()) > 12
        });
    visit.move_for_test(far.expect("a level wide enough"));
    tick(&mut test);
    assert_eq!(
        state(&test, &quest),
        Some(EscortAt::Waiting {
            site: place.0,
            level: place.1,
            spot
        })
    );
    meet(&mut test, &quest);
    // Following, they come to another level with the player.
    assert!(test_enter_poi(&mut test.client, &mut test.world, poi, 0));
    tick(&mut test);
    let here = test.client.visit.as_ref().map(|v| v.population());
    assert_eq!(test.world.escortee_at(mob).map(|at| at.0), here);
    assert_eq!(state(&test, &quest), Some(EscortAt::With));
}

#[test]
fn following_they_walk_after_the_quester() {
    let mut test = TestClient::new();
    let wide = (escort_quests(&mut test).into_iter()).find(|(_, _, poi)| {
        let mut test = TestClient::new();
        assert!(test_enter_poi(&mut test.client, &mut test.world, *poi, 0));
        let site = test.client.visit.as_ref().unwrap().site().unwrap().levels();
        assert!(test_enter_poi(
            &mut test.client,
            &mut test.world,
            *poi,
            site - 1
        ));
        let (_, _, width, height) = test.client.visit.as_ref().unwrap().cells().bounds();
        width.min(height) > 30
    });
    let (quest, _, _) = take_and_go(&mut test, wide.expect("a wide level near the start"));
    meet(&mut test, &quest);
    let mob = escortee_id(&test.client, quest.id);
    let (_, spot) = test.world.escortee_at(mob).unwrap();
    // A few cells off, where it can be walked to.
    let visit = test.client.visit.as_mut().unwrap();
    let (cx, cy) = spot.cell();
    let off = (-8..=8)
        .flat_map(|dy| (-8..=8).map(move |dx| (cx + dx, cy + dy)))
        .filter(|&(x, y)| (x - cx).abs().max((y - cy).abs()) >= 6)
        .find(|&cell| visit.walkable_for_test(cell))
        .expect("somewhere a few cells off");
    visit.move_for_test(off);
    for _ in 0..400 {
        test.world.action_clock.advance(1);
        crate::entities::run_mobs(std::slice::from_mut(&mut test.client), &mut test.world);
        tick(&mut test);
    }
    let (_, now) = test.world.escortee_at(mob).unwrap();
    let (x, y) = now.cell();
    assert!(
        (x - off.0).abs().max((y - off.1).abs()) <= 3,
        "{now:?} near {off:?}"
    );
    assert_eq!(state(&test, &quest), Some(EscortAt::With));
}
