use shared::poi::CHUNK_SIZE;
use shared::quest::{DungeonWay, Hook, Quest, Target};

use super::dungeon::{RESET_AFTER, enter, quest_ended, site_of};
use super::dungeon_reset::reset_due;
use super::progress::{Event, advance};
use super::tests::NOW;
use super::{accept::accept, cache};
use crate::connections::tests::TestClient;
use crate::storage::SiteId;

/// A quest with a cellar of the giver's own, open to anyone, in a town near
/// the start, and that town's poi.
fn a_cellar(test: &mut TestClient) -> (shared::poi::Poi, Quest) {
    a_dungeon(test, DungeonWay::Cellar)
}

/// A quest with a dungeon entered `way`, open to anyone, in a town near the
/// start, and that town's poi.
pub(super) fn a_dungeon(test: &mut TestClient, way: DungeonWay) -> (shared::poi::Poi, Quest) {
    let (x, y) = test.client.world_position();
    let reach = 24 * CHUNK_SIZE;
    let mut towns: Vec<_> = (test.world.pois)
        .in_area((x - reach, y - reach), (x + reach, y + reach))
        .into_iter()
        .filter(|poi| poi.kind.is_town())
        .collect();
    towns.sort_unstable_by_key(|poi| poi.chunk);
    for town in towns {
        let found = cache::town(&mut test.world, town.chunk).expect("a town");
        let mut quests: Vec<Quest> = found.quests.to_vec();
        let givers = shared::quest::daily_givers(&found.poi, &found.buildings);
        for building in givers {
            for day in 0..20 {
                let daily = cache::daily(&mut test.world, town.chunk, building, NOW / 86_400 + day);
                quests.extend(daily.map(|quest| (*quest).clone()));
            }
        }
        let open = quests.into_iter().find(|quest| {
            quest.dungeon.is_some_and(|d| d.way == way) && quest.requirements.is_empty()
        });
        if let Some(quest) = open {
            return (town, quest);
        }
    }
    panic!("no {way:?} quest near the start");
}

/// In the town, then the quest taken, which goes straight into its cellar.
fn taken_and_in(test: &mut TestClient) -> (shared::poi::Poi, Quest, SiteId) {
    let (town, quest) = a_cellar(test);
    assert!(crate::sites::test_enter_poi(
        &mut test.client,
        &mut test.world,
        town,
        0
    ));
    assert!(accept(
        &mut test.client,
        &mut test.world,
        quest.id,
        NOW,
        false
    ));
    let site = site_of(&test.client, quest.id).unwrap();
    (town, quest, site)
}

fn visit_site(test: &TestClient) -> Option<SiteId> {
    test.client.visit.as_ref().map(crate::sites::Visit::id)
}

fn kill_step(quest: &Quest) -> (usize, shared::creature::CreatureKind) {
    (quest.steps.iter().enumerate())
        .find_map(|(i, step)| match step.target {
            Target::Dungeon {
                hook: Hook::Kills(kind),
                ..
            } => Some((i, kind)),
            _ => None,
        })
        .expect("a cellar's kills")
}

#[test]
fn taking_a_cellar_quest_goes_into_a_copy_of_its_own_with_its_creatures() {
    let mut test = TestClient::new();
    let (_, quest, site) = taken_and_in(&mut test);
    assert_eq!(visit_site(&test), Some(site));
    let SiteId::Quest {
        character,
        quest: of,
    } = site
    else {
        panic!("{site:?}");
    };
    assert_eq!(Some(character), test.client.character_id);
    assert_eq!(of, quest.id);
    let dungeon = quest.dungeon.unwrap();
    let population = &test.world.populations.0[&(site, 0)];
    assert!(!population.mobs.is_empty());
    assert!(
        population
            .mobs
            .iter()
            .all(|mob| mob.kind == dungeon.creatures)
    );
    // Kills there count; the same kind elsewhere doesn't.
    let (step, kind) = kill_step(&quest);
    let elsewhere = Event::Killed {
        kind,
        site: Some(quest.id.town),
        dungeon: None,
    };
    advance(&mut test.client, &mut test.world, elsewhere, NOW);
    let counts = |test: &TestClient| test.client.character.quests.taken[&quest.id].counts[step];
    assert_eq!(counts(&test), 0);
    let inside = Event::Killed {
        kind,
        site: None,
        dungeon: Some(quest.id),
    };
    advance(&mut test.client, &mut test.world, inside, NOW);
    assert_eq!(counts(&test), 1);
}

#[test]
fn coming_out_goes_back_and_an_hour_later_it_resets() {
    let mut test = TestClient::new();
    let (town, quest, site) = taken_and_in(&mut test);
    let (step, kind) = kill_step(&quest);
    let inside = Event::Killed {
        kind,
        site: None,
        dungeon: Some(quest.id),
    };
    advance(&mut test.client, &mut test.world, inside, NOW);
    assert!(crate::sites::leave_dungeon(
        &mut test.client,
        &mut test.world
    ));
    // Back in the town (set into the wilderness), not on the world map.
    let visit = test.client.visit.as_ref().expect("back in the town");
    assert!(visit.is_wilderness() || visit.poi().is_some_and(|poi| poi.chunk == town.chunk));
    let left = test.client.character.quests.dungeons[&quest.id]
        .left
        .unwrap();
    // Not yet: the copy and the kill are kept.
    assert!(reset_due(
        &mut test.client,
        &mut test.world,
        left + RESET_AFTER - 1
    ));
    assert!(test.world.sites.contains_key(&site));
    assert_eq!(
        test.client.character.quests.taken[&quest.id].counts[step],
        1
    );
    // An hour on: made afresh, the kill gone (it wasn't all of them).
    assert!(reset_due(
        &mut test.client,
        &mut test.world,
        left + RESET_AFTER
    ));
    assert!(!test.world.sites.contains_key(&site));
    assert_eq!(test.client.character.quests.dungeons[&quest.id].left, None);
    assert_eq!(
        test.client.character.quests.taken[&quest.id].counts[step],
        0
    );
}

#[test]
fn logging_out_inside_keeps_the_copy_with_no_hour_running() {
    let mut test = TestClient::new();
    let (_, quest, site) = taken_and_in(&mut test);
    let saved = test.client.visit.as_ref().unwrap().save();
    assert_eq!(test.client.character.quests.dungeons[&quest.id].left, None);
    // Days later, nothing resets while they are (still) inside.
    assert!(reset_due(
        &mut test.client,
        &mut test.world,
        NOW + 10 * RESET_AFTER
    ));
    assert!(
        test.client
            .character
            .quests
            .dungeons
            .contains_key(&quest.id)
    );
    test.client.visit = None;
    assert!(crate::sites::resume(
        &mut test.client,
        &mut test.world,
        saved
    ));
    assert_eq!(visit_site(&test), Some(site));
}

#[test]
fn giving_up_inside_puts_them_out_and_the_copy_goes() {
    let mut test = TestClient::new();
    let (_, quest, site) = taken_and_in(&mut test);
    assert!(super::give_up::end(
        &mut test.client,
        &mut test.world,
        quest.id,
        false,
        NOW
    ));
    assert_ne!(visit_site(&test), Some(site));
    assert!(
        !test
            .client
            .character
            .quests
            .dungeons
            .contains_key(&quest.id)
    );
    assert!(!test.world.sites.contains_key(&site));
    // Ended quests have no dungeon to go into.
    assert!(enter(&mut test.client, &mut test.world, quest.id, false));
    assert_ne!(visit_site(&test), Some(site));
    assert!(quest_ended(&mut test.client, &mut test.world, quest.id));
}
