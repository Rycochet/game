use shared::creature::CreatureKind;
use shared::item::Inventory;
use shared::poi::{CHUNK_SIZE, ChunkPos};
use shared::protocol::{OfferState, QuestNews, QuestRefusal, QuestState, ServerMessage};
use shared::quest::{Hook, Need, Quest, Target};
use shared::story::RewardPart;

use super::progress::{Event, advance, complete_step};
use super::{accept::accept, cache, give_up::end, offers, turn_in::turn_in};
use crate::connections::tests::TestClient;
use crate::storage::RETAKE_AFTER;

const NOW: i64 = 1_800_000_000;

/// A town near the start with a quest anyone may take that pays coins.
fn a_town(test: &mut TestClient) -> (ChunkPos, Quest) {
    let (x, y) = test.client.world_position();
    let reach = 12 * CHUNK_SIZE;
    let mut towns: Vec<_> = (test
        .world
        .pois
        .in_area((x - reach, y - reach), (x + reach, y + reach)))
    .into_iter()
    .filter(|poi| poi.kind.is_town())
    .map(|poi| poi.chunk)
    .collect();
    towns.sort_unstable();
    for chunk in towns {
        let town = cache::town(&mut test.world, chunk).expect("a town");
        let open = town.quests.iter().find(|quest| {
            quest.requirements.is_empty()
                && quest
                    .reward
                    .iter()
                    .any(|part| matches!(part, RewardPart::Coins(_)))
        });
        if let Some(quest) = open {
            return (chunk, quest.clone());
        }
    }
    panic!("no town near the start with a quest open to all");
}

/// Everything the client has been told about quests since last asked.
fn news(test: &mut TestClient) -> Vec<QuestNews> {
    (test.send(&[]).into_iter())
        .filter_map(|message| match message {
            ServerMessage::Quest(news) => Some(news),
            _ => None,
        })
        .collect()
}

fn copper_carried(inventory: &Inventory) -> u64 {
    (inventory.items.iter())
        .filter_map(|placed| Some(placed.item.kind.coin_value()? * u64::from(placed.item.count)))
        .sum()
}

#[test]
fn a_quest_is_taken_done_and_handed_in_for_its_coins() {
    let mut test = TestClient::new();
    let (town, quest) = a_town(&mut test);
    let id = quest.id;
    let offered = offers::giver_offers(&test.client, &mut test.world, town, quest.building, NOW);
    let offer = offered.iter().find(|offer| offer.quest.id == id);
    assert!(matches!(
        offer.map(|o| &o.state),
        Some(OfferState::Available { .. })
    ));

    assert!(accept(&mut test.client, &mut test.world, id, NOW, false));
    assert!(test.client.character.quests.taken.contains_key(&id));
    for step in &quest.steps {
        let known = test.client.character.pois.get(&step.target.chunk());
        assert!(
            known.is_some_and(|known| known.known && known.named),
            "its targets are known"
        );
    }
    let told = news(&mut test);
    assert!(told.iter().any(|news| matches!(news, QuestNews::Changed(entry) if entry.id == id && entry.state == QuestState::Taken)));

    // Not ready: refused, and kept.
    turn_in(&mut test.client, &mut test.world, id, NOW, false);
    let told = news(&mut test);
    assert!(told.contains(&QuestNews::Refused {
        id,
        why: QuestRefusal::NotReady
    }));

    // No room for the reward: the giver keeps it, and the quest stays ready.
    for step in 0..quest.steps.len() {
        assert!(complete_step(&mut test.client, &mut test.world, id, step));
    }
    let carried = test.client.inventory.clone();
    test.client.inventory.items.clear();
    test.client.inventory.worn.clear();
    test.client.inventory.size = (0, 0);
    turn_in(&mut test.client, &mut test.world, id, NOW, false);
    let told = news(&mut test);
    assert!(told.contains(&QuestNews::Refused {
        id,
        why: QuestRefusal::NoRoom
    }));
    assert!(test.client.character.quests.taken.contains_key(&id));

    test.client.inventory = carried;
    test.client.inventory.items.clear();
    turn_in(&mut test.client, &mut test.world, id, NOW, false);
    let paid: u64 = (quest.reward.iter())
        .map(|part| match part {
            RewardPart::Coins(copper) => *copper,
            _ => 0,
        })
        .sum();
    assert_eq!(copper_carried(&test.client.inventory), paid);
    let book = &test.client.character.quests;
    assert!(!book.taken.contains_key(&id));
    assert_eq!(book.done.get(&id), Some(&NOW));
    assert!(book.standing.get(&town).is_some_and(|&points| points > 0));
    let told = news(&mut test);
    assert!(
        told.iter()
            .any(|news| matches!(news, QuestNews::TurnedIn { id: done, .. } if *done == id))
    );

    // Done once: never offered again.
    let offered = offers::giver_offers(&test.client, &mut test.world, town, quest.building, NOW);
    assert!(offered.iter().all(|offer| offer.quest.id != id));
}

#[test]
fn a_quest_given_up_comes_back_an_hour_later() {
    let mut test = TestClient::new();
    let (town, quest) = a_town(&mut test);
    let id = quest.id;
    accept(&mut test.client, &mut test.world, id, NOW, false);
    complete_step(&mut test.client, &mut test.world, id, 0);
    end(&mut test.client, &mut test.world, id, false, NOW);
    assert!(!test.client.character.quests.taken.contains_key(&id));
    let soon = offers::giver_offers(
        &test.client,
        &mut test.world,
        town,
        quest.building,
        NOW + 60,
    );
    assert!(soon.iter().all(|offer| offer.quest.id != id));
    let later = NOW + RETAKE_AFTER;
    assert!(offers::on_offer(&test.client, &mut test.world, id, quest.building, later).is_some());
    accept(&mut test.client, &mut test.world, id, later, false);
    let progress = &test.client.character.quests.taken[&id];
    assert!(
        progress.counts.iter().all(|&count| count == 0),
        "progress starts again"
    );
}

#[test]
fn kills_and_arrivals_move_steps_on() {
    let mut test = TestClient::new();
    let (x, y) = test.client.world_position();
    let reach = 12 * CHUNK_SIZE;
    let mut towns: Vec<_> = (test
        .world
        .pois
        .in_area((x - reach, y - reach), (x + reach, y + reach)))
    .into_iter()
    .filter(|poi| poi.kind.is_town())
    .map(|poi| poi.chunk)
    .collect();
    towns.sort_unstable();
    let quests: Vec<Quest> = (towns.into_iter())
        .flat_map(|chunk| cache::town(&mut test.world, chunk).unwrap().quests.to_vec())
        .collect();
    let kills = quests.iter().find_map(|quest| {
        let step = quest.steps.iter().position(|step| {
            step.after.is_empty()
                && matches!(
                    step.target,
                    Target::Poi {
                        hook: Hook::Kills(_),
                        ..
                    }
                )
        })?;
        Some((quest.clone(), step))
    });
    let (quest, step) = kills.expect("a quest with kills to start with");
    let Target::Poi {
        chunk,
        hook: Hook::Kills(kind),
    } = quest.steps[step].target
    else {
        unreachable!();
    };
    test.client.character.quests.taken.insert(
        quest.id,
        crate::storage::Progress {
            at: NOW,
            counts: vec![0; quest.steps.len()],
        },
    );
    let other = if kind == CreatureKind::Kobold {
        CreatureKind::Wolf
    } else {
        CreatureKind::Kobold
    };
    advance(
        &mut test.client,
        &mut test.world,
        Event::Killed {
            kind: other,
            site: Some(chunk),
        },
        NOW,
    );
    advance(
        &mut test.client,
        &mut test.world,
        Event::Killed { kind, site: None },
        NOW,
    );
    assert_eq!(
        test.client.character.quests.taken[&quest.id].counts[step],
        0
    );
    advance(
        &mut test.client,
        &mut test.world,
        Event::Killed {
            kind,
            site: Some(chunk),
        },
        NOW,
    );
    assert_eq!(
        test.client.character.quests.taken[&quest.id].counts[step],
        1
    );
    let most = quest.steps[step].count.unwrap_or(1);
    for _ in 0..most + 3 {
        advance(
            &mut test.client,
            &mut test.world,
            Event::Killed {
                kind,
                site: Some(chunk),
            },
            NOW,
        );
    }
    assert_eq!(
        test.client.character.quests.taken[&quest.id].counts[step], most,
        "never past the count"
    );
}

#[test]
fn a_quest_needing_another_alignment_is_never_shown() {
    let mut test = TestClient::new();
    let (x, y) = test.client.world_position();
    let reach = 40 * CHUNK_SIZE;
    let mut towns: Vec<_> = (test
        .world
        .pois
        .in_area((x - reach, y - reach), (x + reach, y + reach)))
    .into_iter()
    .filter(|poi| poi.kind.is_town())
    .map(|poi| poi.chunk)
    .collect();
    towns.sort_unstable();
    let dark = towns.into_iter().find_map(|chunk| {
        let town = cache::town(&mut test.world, chunk).unwrap();
        town.quests
            .iter()
            .find(|quest| quest.is_hidden_by_alignment() && quest.id.earlier().is_none())
            .cloned()
    });
    let quest = dark.expect("a dark chain somewhere near");
    let Some(Need::Alignment(band)) = (quest.requirements.iter())
        .map(|requirement| requirement.need)
        .find(|need| matches!(need, Need::Alignment(_)))
    else {
        unreachable!();
    };
    let good = shared::alignment::Alignment::new(1000);
    test.client.sheet.alignment = Some(good);
    assert_ne!(good.band(), band);
    let offered = offers::giver_offers(
        &test.client,
        &mut test.world,
        quest.id.town,
        quest.building,
        NOW,
    );
    assert!(
        offered.iter().all(|offer| offer.quest.id != quest.id),
        "hidden, not greyed"
    );
}
