//! Dailies a day apart and kept until handed in; cities' artifact lines,
//! open to the Honoured, paying the planet's set item once and artifacts
//! after.

use shared::item::Quality;
use shared::poi::{CHUNK_SIZE, ChunkPos, PoiKind};
use shared::protocol::OfferState;
use shared::quest::{Quest, QuestKind, SECONDS_PER_DAY, Slot, daily_givers, set_item_of, utc_day};
use shared::story::RewardPart;

use super::progress::complete_step;
use super::tests::NOW;
use super::{accept::accept, cache, offers, prize, turn_in::turn_in};
use crate::connections::tests::TestClient;

/// The nearest town to the start (by chunk order) with a daily giver.
fn a_daily_giver(test: &mut TestClient) -> (ChunkPos, u16) {
    for chunk in towns_near(test, 12) {
        let town = cache::town(&mut test.world, chunk).expect("a town");
        if let Some(&building) = daily_givers(&town.poi, &town.buildings).first() {
            return (chunk, building);
        }
    }
    panic!("no daily giver near the start");
}

/// A city near the start with an artifact line, and the line.
fn a_line(test: &mut TestClient) -> (ChunkPos, Vec<Quest>) {
    for chunk in towns_near(test, 48) {
        let town = cache::town(&mut test.world, chunk).expect("a town");
        if town.poi.kind != PoiKind::City {
            continue;
        }
        let line: Vec<Quest> = (town.quests.iter())
            .filter(|quest| quest.kind == QuestKind::Artifact)
            .cloned()
            .collect();
        if !line.is_empty() {
            return (chunk, line);
        }
    }
    panic!("no city near the start with an artifact line");
}

fn towns_near(test: &TestClient, chunks: i64) -> Vec<ChunkPos> {
    let (x, y) = test.client.world_position();
    let reach = chunks * CHUNK_SIZE;
    let mut towns: Vec<ChunkPos> = (test.world.pois)
        .in_area((x - reach, y - reach), (x + reach, y + reach))
        .into_iter()
        .filter(|poi| poi.kind.is_town())
        .map(|poi| poi.chunk)
        .collect();
    towns.sort_unstable();
    towns
}

fn the_daily(test: &mut TestClient, town: ChunkPos, building: u16, now: i64) -> Option<Quest> {
    (offers::giver_offers(&test.client, &mut test.world, town, building, now).into_iter())
        .map(|offer| offer.quest)
        .find(|quest| matches!(quest.id.slot, Slot::Daily { .. }))
}

fn finish(test: &mut TestClient, quest: &Quest, now: i64) {
    for step in 0..quest.steps.len() {
        assert!(complete_step(
            &mut test.client,
            &mut test.world,
            quest.id,
            step
        ));
    }
    test.client.inventory.items.clear();
    turn_in(&mut test.client, &mut test.world, quest.id, now, false);
}

#[test]
fn a_daily_is_kept_until_handed_in_and_the_next_comes_a_day_later() {
    let mut test = TestClient::new();
    let (town, building) = a_daily_giver(&mut test);
    let today = the_daily(&mut test, town, building, NOW).expect("today's daily");
    assert_eq!(
        today.id.slot,
        Slot::Daily {
            building,
            day: utc_day(NOW)
        }
    );
    assert!(accept(
        &mut test.client,
        &mut test.world,
        today.id,
        NOW,
        false
    ));

    // Days later, the one taken is still the one offered.
    let later = NOW + 3 * SECONDS_PER_DAY;
    let kept = the_daily(&mut test, town, building, later).expect("the daily taken");
    assert_eq!(kept.id, today.id);

    // Handed in: nothing until a whole day has gone by, then that day's.
    finish(&mut test, &today, later);
    assert!(!test.client.character.quests.taken.contains_key(&today.id));
    assert!(the_daily(&mut test, town, building, later + SECONDS_PER_DAY - 1).is_none());
    let next = the_daily(&mut test, town, building, later + SECONDS_PER_DAY).expect("the next");
    let day = utc_day(later + SECONDS_PER_DAY);
    assert_eq!(next.id.slot, Slot::Daily { building, day });
}

#[test]
fn an_artifact_line_opens_to_the_honoured() {
    let mut test = TestClient::new();
    let (city, line) = a_line(&mut test);
    let first = &line[0];
    let state = |test: &mut TestClient| {
        let offers = offers::giver_offers(&test.client, &mut test.world, city, first.building, NOW);
        (offers.into_iter())
            .find(|offer| offer.quest.id == first.id)
            .map(|offer| offer.state)
    };
    assert!(matches!(state(&mut test), Some(OfferState::Locked(_))));
    test.client.character.quests.standing.insert(city, 75);
    assert!(matches!(
        state(&mut test),
        Some(OfferState::Available { .. })
    ));
}

#[test]
fn the_set_item_is_found_once_then_artifacts_are_paid() {
    let mut test = TestClient::new();
    let (_, line) = a_line(&mut test);
    let mut last = line.last().unwrap().clone();
    // Whichever this city's line ends in, ask for the set item.
    for part in &mut last.reward {
        if *part == RewardPart::Artifact {
            *part = RewardPart::SetItem;
        }
    }
    let (items, set_item) = prize::prize_items(&test.client, &test.world, &last);
    let (kind, quality) = set_item_of(test.world.seed.value());
    assert!(set_item);
    assert!(
        items
            .iter()
            .any(|item| item.set && item.kind == kind && item.quality == Some(quality))
    );

    // Found by another quest: this one now pays an artifact, in its words too.
    let mut other = last.id;
    other.slot = Slot::Artifact { step: 99 };
    test.client.character.quests.set_item = Some(other);
    let seen = prize::as_seen(&test.client.character.quests, last.clone());
    assert!(!seen.reward.contains(&RewardPart::SetItem));
    assert!(!seen.story.facts.reward.contains(&RewardPart::SetItem));
    let (items, set_item) = prize::prize_items(&test.client, &test.world, &last);
    assert!(!set_item);
    assert!(items.iter().all(|item| !item.set));
    assert!(
        items
            .iter()
            .any(|item| item.quality == Some(Quality::Artifact))
    );
}

#[test]
fn handing_in_the_line_pays_its_prize() {
    let mut test = TestClient::new();
    let (_, line) = a_line(&mut test);
    let last = line.last().unwrap().clone();
    let progress = crate::storage::Progress {
        at: NOW,
        counts: vec![0; last.steps.len()],
        ready: false,
        escort: None,
    };
    (test.client.character.quests.taken).insert(last.id, progress);
    finish(&mut test, &last, NOW);
    let book = &test.client.character.quests;
    assert!(book.done.contains_key(&last.id));
    let carried = &test.client.inventory.items;
    let prize = carried.iter().find(|placed| placed.item.quality.is_some());
    let prize = prize.expect("its prize, besides coins").item.clone();
    if last.reward.contains(&RewardPart::SetItem) {
        assert!(prize.set);
        assert_eq!(book.set_item, Some(last.id));
    } else {
        assert_eq!(prize.quality, Some(Quality::Artifact));
        assert_eq!(book.set_item, None);
    }
}
