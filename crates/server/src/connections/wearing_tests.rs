use shared::item::{Favour, Item, ItemKind, Quality};
use shared::protocol::{ClientMessage, ServerMessage};
use shared::wear::WearRefused;

use super::tests::TestClient;

fn hat(seed: u64, favour: Option<Favour>) -> ClientMessage {
    ClientMessage::DevGiveItem {
        kind: ItemKind::Hat,
        size: None,
        magical: false,
        quality: Quality::Rare,
        set: false,
        favour,
        seed: Some(seed),
        count: 1,
        identical: true,
        request: shared::item_form::RollRequest::NONE,
        to: None,
    }
}

/// The carried hats' ids, in the order they are carried.
fn hats(test: &TestClient) -> Vec<shared::id::Id> {
    let carried = test
        .client
        .inventory
        .items
        .iter()
        .map(|placed| &placed.item);
    carried
        .filter(|item: &&Item| item.kind == ItemKind::Hat)
        .map(|item| item.id)
        .collect()
}

#[test]
fn wearing_a_second_hat_takes_the_first_off_into_the_inventory() {
    let mut test = TestClient::new();
    test.send(&[hat(1, None)]);
    let first = hats(&test)[0];
    test.send(&[ClientMessage::WearItem { item: first }]);
    test.send(&[hat(2, None)]);
    let second = hats(&test)[0];
    test.send(&[ClientMessage::WearItem { item: second }]);
    assert!(test.client.inventory.is_worn(second));
    assert_eq!(hats(&test), [first]);
}

#[test]
fn a_cursed_hat_holds_and_the_player_is_told_why() {
    let mut test = TestClient::new();
    test.send(&[hat(1, Some(Favour::Cursed))]);
    let cursed = hats(&test)[0];
    test.send(&[ClientMessage::WearItem { item: cursed }]);
    test.send(&[hat(2, None)]);
    let other = hats(&test)[0];
    let answers = test.send(&[ClientMessage::WearItem { item: other }]);
    assert!(answers.contains(&ServerMessage::NotWorn {
        item: other,
        why: WearRefused::Stuck(cursed),
    }));
    assert!(test.client.inventory.is_worn(cursed));
    assert_eq!(hats(&test), [other]);
}
