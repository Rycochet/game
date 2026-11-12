use crate::id::Id;
use crate::item::{Inventory, Item, ItemKind, Placed, Rotation, StackLimits};

fn item(n: u128, kind: ItemKind) -> Item {
    Item::new(Id::for_test(n), kind)
}

#[test]
fn items_are_found_inside_bags_and_in_stacks() {
    let mut bag = item(1, ItemKind::Backpack);
    let mut stack = item(2, ItemKind::Ring);
    stack.above.push(item(3, ItemKind::Ring));
    bag.contents.push(Placed {
        item: stack,
        x: 0,
        y: 0,
        rotation: Rotation::None,
    });
    for n in 1..=3 {
        assert_eq!(
            bag.find_within(Id::for_test(n)).map(|i| i.id),
            Some(Id::for_test(n))
        );
    }
    assert!(bag.find_within(Id::for_test(4)).is_none());
}

#[test]
fn an_item_alone_holds_nothing() {
    let mut stack = item(2, ItemKind::Ring);
    stack.above.push(item(3, ItemKind::Ring));
    let alone = stack.alone();
    assert!(alone.above.is_empty() && alone.contents.is_empty());
    assert_eq!(alone.id, stack.id);
}

#[test]
fn an_inventory_finds_what_is_stacked_worn_or_binned() {
    let mut inventory = Inventory::default();
    let mut stack = item(2, ItemKind::Ring);
    stack.above.push(item(3, ItemKind::Ring));
    inventory
        .give(stack, StackLimits::NONE, &mut crate::id::stand_ins())
        .unwrap();
    inventory.bin = Some(item(4, ItemKind::Pouch));
    for n in 2..=4 {
        assert!(inventory.find_within(Id::for_test(n)).is_some(), "{n}");
    }
    assert!(inventory.find_within(Id::for_test(5)).is_none());
}
