use shared::id::Id;
use shared::item::{Holder, Inventory, Item, ItemKind, Quality, Rotation};
use shared::poi::PoiKind;
use shared::protocol::{ClientMessage, ServerMessage};

use crate::connections::tests::TestClient;

fn give_coins(count: u32) -> ClientMessage {
    ClientMessage::DevGiveItem(shared::protocol::DevGive {
        kind: ItemKind::CopperCoin,
        size: None,
        magical: false,
        quality: Quality::Rare,
        set: false,
        favour: None,
        seed: Some(1),
        count,
        identical: true,
        request: shared::item_form::RollRequest::NONE,
        to: None,
    })
}

fn inventory(answers: &[ServerMessage]) -> Inventory {
    let found = answers.iter().rev().find_map(|answer| match answer {
        ServerMessage::Inventory(inventory) => Some(inventory.clone()),
        _ => None,
    });
    found.expect("an inventory among the answers")
}

/// Part of the stack, moved to `x` in the inventory by the client's stand-in.
fn move_part(stack: Id, count: u32, x: u8) -> ClientMessage {
    ClientMessage::PartOfStack {
        stack,
        count,
        then: Box::new(ClientMessage::MoveItem {
            item: Id::stand_in(1),
            to: Holder::Inventory,
            x,
            y: 0,
            rotation: Rotation::None,
        }),
    }
}

#[test]
fn part_of_a_counted_stack_moves_with_a_new_id_from_the_server() {
    let mut test = TestClient::new();
    let carried = inventory(&test.send(&[give_coins(30)]));
    let coins = carried.items[0].item.clone();
    assert_eq!((coins.count(), coins.above.len()), (30, 0), "a number");
    // Back onto its own place: refused, and it goes back on the stack.
    let carried = inventory(&test.send(&[move_part(coins.id, 12, coins_x(&carried))]));
    assert_eq!(carried.items.len(), 1);
    assert_eq!(carried.find(coins.id).map(Item::count), Some(30));
    // Elsewhere: twelve go, named by the server.
    let x = coins_x(&carried) + 3;
    let carried = inventory(&test.send(&[move_part(coins.id, 12, x)]));
    assert_eq!(carried.find(coins.id).map(Item::count), Some(18));
    let part = carried.items.iter().find(|placed| placed.x == x).unwrap();
    assert_eq!(part.item.count(), 12);
    assert!(part.item.id != coins.id && !part.item.id.is_stand_in());
    // More than there are: refused.
    let carried = inventory(&test.send(&[move_part(coins.id, 19, 0)]));
    assert_eq!(carried.find(coins.id).map(Item::count), Some(18));
}

#[test]
fn part_of_a_counted_stack_is_dropped_on_its_own() {
    let mut test = TestClient::new();
    test.send(&[ClientMessage::DevEnterSite {
        kind: PoiKind::Castle,
        seed: 1,
        level: 0,
        ruin: 0,
    }]);
    let carried = inventory(&test.send(&[give_coins(30)]));
    let coins = carried.items[0].item.id;
    let drop = ClientMessage::PartOfStack {
        stack: coins,
        count: 10,
        then: Box::new(ClientMessage::DropItem {
            item: Id::stand_in(1),
        }),
    };
    let answers = test.send(&[drop]);
    assert_eq!(inventory(&answers).find(coins).map(Item::count), Some(20));
    let ground = answers.iter().rev().find_map(|answer| match answer {
        ServerMessage::GroundItems { items, .. } => Some(items.clone()),
        _ => None,
    });
    let ground = ground.expect("what is on the ground");
    assert_eq!(ground[0].count(), 10);
    assert!(!ground[0].id.is_stand_in());
}

/// Where the first stack lies across.
fn coins_x(carried: &Inventory) -> u8 {
    carried.items[0].x
}
