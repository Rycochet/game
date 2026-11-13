//! A poi's unique quest item: lying on the site's deepest level for each
//! character still wanting it, picked up from the ground like anything else,
//! then remembered and offered as already found, and gone for them.

use shared::item::{Holder, Rotation};
use shared::poi::{CHUNK_SIZE, ChunkPos, Poi};
use shared::protocol::{Ahead, ClientMessage, OfferState, ServerMessage};
use shared::quest::{Hook, Quest, Target};

use super::finds::find_here;
use super::tests::NOW;
use super::{cache, offers};
use crate::connections::tests::TestClient;
use crate::sites::test_enter_poi;

/// A quest near the start with an item to find, and the item's poi.
fn an_item_quest(test: &mut TestClient) -> (Quest, Poi) {
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
            let item = quest.steps.iter().find_map(|step| match step.target {
                Target::Poi {
                    chunk,
                    hook: Hook::Item,
                } => Some(chunk),
                _ => None,
            });
            if let Some(at) = item {
                let poi = test.world.pois.in_chunk(at).expect("its poi");
                return (quest.clone(), poi);
            }
        }
    }
    panic!("no quest near the start with an item to find");
}

/// Into the poi's deepest level.
fn to_the_bottom(test: &mut TestClient, poi: Poi) {
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
}

#[test]
fn the_item_lies_on_the_deepest_level_for_whoever_still_wants_it() {
    let mut test = TestClient::new();
    let (quest, poi) = an_item_quest(&mut test);
    to_the_bottom(&mut test, poi);
    let find = find_here(&test.client, &mut test.world).expect("it lies here");
    assert_eq!(find.item.quest, Some(quest.id));
    assert_eq!(find.quest.id, quest.id);
    // The same each time for one character.
    let again = find_here(&test.client, &mut test.world).unwrap();
    assert_eq!((again.at, again.item.id), (find.at, find.item.id));

    // Not for one who found it already.
    test.client.character.quests.found.insert(quest.id);
    assert!(find_here(&test.client, &mut test.world).is_none());
}

#[test]
fn picked_up_it_is_remembered_and_offered_as_found() {
    let mut test = TestClient::new();
    let (quest, poi) = an_item_quest(&mut test);
    to_the_bottom(&mut test, poi);
    let find = find_here(&test.client, &mut test.world).unwrap();
    // Standing where it lies, it is told of as on the ground.
    let visit = test.client.visit.as_mut().unwrap();
    visit.move_for_test(find.at.cell());
    let answers = test.send(&[]);
    let told = answers.iter().any(|message| match message {
        ServerMessage::GroundItems { items, .. } => items.iter().any(|i| i.id == find.item.id),
        _ => false,
    });
    assert!(told, "it lies in reach");

    let (x, y, rotation) = (test.client.inventory)
        .first_place_for_new(&find.item, Holder::Inventory, Rotation::None)
        .expect("room for it");
    test.send(&[ClientMessage::TakeFromGround {
        item: find.item.id,
        to: Holder::Inventory,
        x,
        y,
        rotation,
    }]);
    assert!(
        test.client.inventory.find(find.item.id).is_some(),
        "carried"
    );
    let book = &test.client.character.quests;
    assert!(book.found.contains(&quest.id));
    assert!(find_here(&test.client, &mut test.world).is_none());

    let (town, building) = (quest.id.town, quest.building);
    let offered = offers::giver_offers(&test.client, &mut test.world, town, building, NOW);
    let state = offered.iter().find(|offer| offer.quest.id == quest.id);
    match state.map(|offer| &offer.state) {
        Some(OfferState::Available { ahead, .. }) => assert_eq!(*ahead, Some(Ahead::Found)),
        Some(OfferState::Locked(hints)) => {
            assert!(hints.contains(&shared::protocol::Hint::Ahead(Ahead::Found)))
        }
        // A later quest of a chain: offered once the one before is done.
        None => {}
        other => panic!("not offered as found: {other:?}"),
    }
}
