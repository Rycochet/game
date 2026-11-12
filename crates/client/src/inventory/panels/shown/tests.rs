use shared::id::Id;
use shared::item::{Holder, Inventory, Item, ItemKind, Rotation, StackLimits};

use super::{grows, shown_size};

/// An inventory with a bag of this kind in it, id 1.
fn with_bag(kind: ItemKind) -> Inventory {
    let mut inventory = Inventory::default();
    let bag = Item::new(Id::for_test(1), kind);
    inventory
        .give(bag, StackLimits::NONE, &mut shared::id::stand_ins())
        .unwrap();
    inventory
}

#[test]
fn an_ordinary_bag_is_drawn_whole() {
    let inventory = with_bag(ItemKind::Backpack);
    let bag = Holder::Inside(Id::for_test(1));
    assert_eq!(shown_size(&inventory, bag), inventory.grid_size(bag));
    assert!(!grows(&inventory, bag) && !grows(&inventory, Holder::Inventory));
}

#[test]
fn a_bag_of_holding_shows_what_is_used_and_room_below() {
    let mut inventory = with_bag(ItemKind::BagOfHolding);
    let bag = Holder::Inside(Id::for_test(1));
    assert!(grows(&inventory, bag));
    // Empty: room for the largest thing it takes, which is smaller than its outside.
    assert_eq!(shown_size(&inventory, bag), Some((8, 3)));
    let apple = Item::new(Id::for_test(2), ItemKind::Apple);
    inventory.add(apple, bag, 0, 9, Rotation::None).unwrap();
    assert_eq!(shown_size(&inventory, bag), Some((8, 13)));
    // Never more than it can ever have.
    let deep = Item::new(Id::for_test(3), ItemKind::Apple);
    inventory.add(deep, bag, 0, 59, Rotation::None).unwrap();
    assert_eq!(shown_size(&inventory, bag), Some((8, 60)));
}
