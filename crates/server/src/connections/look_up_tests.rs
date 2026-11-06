use shared::id::Id;
use shared::item::{Favour, ItemKind};
use shared::protocol::{ClientMessage, MOST_LOOKED_UP, ServerMessage};
use shared::world::WorldSeed;

use super::look_up::answer_look_ups;
use super::tests::{TestClient, given};
use crate::{ServerWorld, ServerWorlds};

/// Asks after these items, and reads the answer.
fn look_up(test: &mut TestClient, ids: &[Id]) -> (Vec<shared::item::Item>, Vec<Id>) {
    test.send(&[ClientMessage::LookUpItems(ids.to_vec())]);
    let world = std::mem::replace(&mut test.world, ServerWorld::new(WorldSeed::new(2)));
    let mut worlds = ServerWorlds::with_home(world);
    answer_look_ups(std::slice::from_mut(&mut test.client), &mut worlds);
    test.world = worlds.into_home();
    let answers: Vec<_> = std::iter::from_fn(|| test.end.receive().unwrap()).collect();
    match answers.as_slice() {
        [ServerMessage::ItemsLookedUp { found, unknown }] => (found.clone(), unknown.clone()),
        other => panic!("one answer, not {other:?}"),
    }
}

#[test]
fn what_is_carried_is_found_alone_and_keeps_its_secrets() {
    let mut test = TestClient::new();
    let hat = given(&mut test, ItemKind::Hat, None, true, Some(Favour::Cursed));
    test.send(&[ClientMessage::WearItem { item: hat }]);
    let bag = given(&mut test, ItemKind::Backpack, None, false, None);
    let nowhere = Id::for_test(999);
    let (found, unknown) = look_up(&mut test, &[hat, nowhere, hat, bag]);
    assert_eq!(
        found.iter().map(|item| item.id).collect::<Vec<_>>(),
        [hat, bag]
    );
    assert_eq!(unknown, [nowhere]);
    assert_eq!(found[0].favour, None, "a secret stays one");
    assert!(
        found
            .iter()
            .all(|item| item.contents.is_empty() && item.above.is_empty())
    );
}

#[test]
fn what_lies_on_the_ground_is_found() {
    let mut test = TestClient::new();
    test.enter_a_castle();
    let ring = given(&mut test, ItemKind::Ring, None, false, None);
    test.send(&[ClientMessage::DropItem { item: ring }]);
    assert!(test.client.inventory.find(ring).is_none());
    let (found, _) = look_up(&mut test, &[ring]);
    assert_eq!(found.len(), 1);
}

#[test]
fn only_so_many_are_looked_up_at_once() {
    let mut test = TestClient::new();
    let ids: Vec<Id> = (0..MOST_LOOKED_UP as u128 + 10).map(Id::for_test).collect();
    let (found, unknown) = look_up(&mut test, &ids);
    assert_eq!(found.len() + unknown.len(), MOST_LOOKED_UP);
}
