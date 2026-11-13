//! Quest items on the server: a delivery's parcel handed over on taking it and
//! needed to deliver, quest items gone when the quest ends, and never given or
//! left on the ground.

use shared::item::{Holder, Item, ItemKind, Rotation};
use shared::poi::{CHUNK_SIZE, ChunkPos};
use shared::quest::{Quest, Target};

use super::progress::{Event, advance};
use super::tests::NOW;
use super::{accept::accept, cache, give_up::end};
use crate::connections::tests::TestClient;
use crate::sites::QuestPlace;

/// A delivery near the start that anyone may take, and the town it goes to.
fn a_delivery(test: &mut TestClient) -> (Quest, ChunkPos) {
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
        for quest in town.quests.iter().filter(|q| q.requirements.is_empty()) {
            let to = quest.steps.iter().find_map(|step| match step.target {
                Target::Town { chunk } => Some(chunk),
                _ => None,
            });
            if let Some(to) = to {
                return (quest.clone(), to);
            }
        }
    }
    panic!("no delivery near the start open to all");
}

fn arrive_in(test: &mut TestClient, town: ChunkPos) {
    let poi = test.world.pois.in_chunk(town).expect("the town");
    let place = QuestPlace {
        poi,
        level: 0,
        deepest: true,
    };
    advance(
        &mut test.client,
        &mut test.world,
        Event::Arrived(place),
        NOW,
    );
}

fn parcels(test: &TestClient, quest: &Quest) -> u32 {
    (test.client.inventory).quest_kind(quest.id, Some(ItemKind::Parcel))
}

#[test]
fn a_delivery_hands_over_its_parcel_which_arriving_delivers() {
    let mut test = TestClient::new();
    let (quest, to) = a_delivery(&mut test);
    assert!(accept(
        &mut test.client,
        &mut test.world,
        quest.id,
        NOW,
        false
    ));
    assert!(test.client.character.quests.taken.contains_key(&quest.id));
    assert_eq!(parcels(&test, &quest), 1, "its parcel is carried");

    arrive_in(&mut test, to);
    let step = quest
        .steps
        .iter()
        .position(|s| s.target == Target::Town { chunk: to });
    let counts = &test.client.character.quests.taken[&quest.id].counts;
    assert_eq!(counts[step.unwrap()], 1, "delivered");
    assert_eq!(parcels(&test, &quest), 0, "handed over");
}

#[test]
fn arriving_without_the_parcel_delivers_nothing() {
    let mut test = TestClient::new();
    let (quest, to) = a_delivery(&mut test);
    assert!(accept(
        &mut test.client,
        &mut test.world,
        quest.id,
        NOW,
        false
    ));
    test.client.inventory.remove_quest_items(quest.id);
    arrive_in(&mut test, to);
    let counts = &test.client.character.quests.taken[&quest.id].counts;
    assert!(counts.iter().all(|&count| count == 0));
}

#[test]
fn a_delivery_with_no_room_for_its_parcel_is_refused() {
    let mut test = TestClient::new();
    let (quest, _) = a_delivery(&mut test);
    // Every cell taken.
    let (wide, high) = test.client.inventory.grid_size(Holder::Inventory).unwrap();
    for y in 0..high {
        for x in 0..wide {
            let stone = Item::new(crate::ids::new_id(), ItemKind::Key);
            let _ = (test.client.inventory).add(stone, Holder::Inventory, x, y, Rotation::None);
        }
    }
    assert!(accept(
        &mut test.client,
        &mut test.world,
        quest.id,
        NOW,
        false
    ));
    assert!(!test.client.character.quests.taken.contains_key(&quest.id));
    assert_eq!(parcels(&test, &quest), 0);
}

#[test]
fn giving_up_a_quest_takes_its_items_away() {
    let mut test = TestClient::new();
    let (quest, _) = a_delivery(&mut test);
    assert!(accept(
        &mut test.client,
        &mut test.world,
        quest.id,
        NOW,
        false
    ));
    assert_eq!(parcels(&test, &quest), 1);
    assert!(end(&mut test.client, &mut test.world, quest.id, false, NOW));
    assert_eq!(parcels(&test, &quest), 0);
}
