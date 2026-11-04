use super::*;
use crate::id::Id;

fn id(n: u128) -> Id {
    Id::for_test(n)
}

/// `count` items of this kind, ids from `first`, the first at the bottom.
fn stack(first: u128, kind: ItemKind, count: u128) -> Item {
    let mut bottom = Item::new(id(first), kind);
    for n in 1..count {
        bottom.put_on(Item::new(id(first + n), kind));
    }
    bottom
}

fn ids(item: &Item) -> Vec<Id> {
    item.members().map(|one| one.id).collect()
}

#[test]
fn given_apples_stack_until_the_weight_limit() {
    let limits = StackLimits::default();
    let mut inventory = Inventory::default();
    for n in 1..=14 {
        inventory
            .give(Item::new(id(n), ItemKind::Apple), limits)
            .unwrap();
    }
    // 13 apples of 150 g make 1950 g, under 2000 g for their one cell.
    assert_eq!(inventory.items.len(), 2);
    let first = &inventory.items[0].item;
    assert_eq!((first.id, first.count()), (id(1), 13));
    assert_eq!(first.top().id, id(13));
    assert_eq!(first.total_weight(), 13 * 150);
    assert_eq!(inventory.items[1].item.count(), 1);
}

#[test]
fn light_things_stack_until_the_count_limit() {
    let limits = StackLimits::default();
    let mut inventory = Inventory::default();
    for n in 1..=100 {
        inventory
            .give(Item::new(id(n), ItemKind::Coin), limits)
            .unwrap();
    }
    let counts: Vec<u32> = inventory.items.iter().map(|p| p.item.count()).collect();
    assert_eq!(counts, [99, 1]);
}

#[test]
fn only_what_looks_the_same_stacks() {
    let apple = Item::new(id(1), ItemKind::Apple);
    assert!(apple.looks_like(&Item::new(id(2), ItemKind::Apple)));
    assert!(!apple.looks_like(&Item::new(id(3), ItemKind::Coin)));
    let hat = |n, quality| Item {
        quality: Some(quality),
        ..Item::new(id(n), ItemKind::Hat)
    };
    assert!(!hat(4, Quality::Common).looks_like(&hat(5, Quality::Uncommon)));
    // A curse nobody knows of can't tell them apart: one that is known can.
    let mut cursed = hat(6, Quality::Common);
    cursed.favour = Some(Favour::Cursed);
    assert!(cursed.looks_like(&hat(7, Quality::Common)));
    cursed.known.favour = true;
    assert!(!cursed.looks_like(&hat(7, Quality::Common)));
    // When something was made isn't seen.
    let mut later = Item::new(id(8), ItemKind::Apple);
    later.made_at = Some(MadeAt {
        world: crate::world::WorldSeed::new(1).id(),
        minute: 99,
    });
    assert!(apple.looks_like(&later));
    // Bags never stack, even empty.
    let pouch = Item::new(id(9), ItemKind::Pouch);
    assert!(!pouch.looks_like(&Item::new(id(10), ItemKind::Pouch)));
}

#[test]
fn a_stack_goes_on_another_as_far_as_it_fits() {
    let limits = StackLimits::default();
    let mut inventory = Inventory::default();
    let (big, small) = (stack(1, ItemKind::Apple, 12), stack(20, ItemKind::Apple, 3));
    inventory
        .add(big, Holder::Inventory, 0, 0, Rotation::None)
        .unwrap();
    inventory
        .add(small, Holder::Inventory, 1, 0, Rotation::None)
        .unwrap();
    assert_eq!(inventory.stack_onto(id(20), id(1), limits), Ok(1));
    // The top went first; what is left keeps its bottom, and so its id.
    let big = inventory.find(id(1)).unwrap();
    assert_eq!((big.count(), big.top().id), (13, id(22)));
    assert_eq!(ids(inventory.find(id(20)).unwrap()), [id(20), id(21)]);
    // Full now.
    assert_eq!(
        inventory.stack_onto(id(20), id(1), limits),
        Err(MoveRefused::NoStackRoom)
    );
}

#[test]
fn a_whole_stack_joins_another_and_goes() {
    let limits = StackLimits::default();
    let mut inventory = Inventory::default();
    let coins = Item::new(id(3), ItemKind::Coin);
    inventory
        .add(
            stack(1, ItemKind::Coin, 2),
            Holder::Inventory,
            0,
            0,
            Rotation::None,
        )
        .unwrap();
    inventory
        .add(coins, Holder::Inventory, 2, 0, Rotation::None)
        .unwrap();
    assert_eq!(inventory.stack_onto(id(1), id(3), limits), Ok(2));
    assert!(inventory.find(id(1)).is_none());
    assert_eq!(ids(inventory.find(id(3)).unwrap()), [id(3), id(1), id(2)]);
}

#[test]
fn something_not_carried_joins_a_stack_and_the_rest_is_given_back() {
    let limits = StackLimits {
        grams_per_cell: 2000,
        most: 4,
    };
    let mut inventory = Inventory::default();
    inventory
        .add(
            stack(1, ItemKind::Coin, 2),
            Holder::Inventory,
            0,
            0,
            Rotation::None,
        )
        .unwrap();
    let rest = inventory
        .add_to_stack(stack(10, ItemKind::Coin, 3), id(1), limits)
        .unwrap();
    assert_eq!(ids(&rest.unwrap()), [id(10)]);
    assert_eq!(inventory.find(id(1)).unwrap().count(), 4);
    let apple = Item::new(id(20), ItemKind::Apple);
    assert_eq!(
        inventory.add_to_stack(apple, id(1), limits),
        Err(MoveRefused::NoStackRoom)
    );
}

#[test]
fn a_bag_counts_one_item_of_a_stack_against_its_most_for_each() {
    let mut inventory = Inventory::default();
    inventory
        .give(Item::new(id(1), ItemKind::Pouch), StackLimits::NONE)
        .unwrap();
    let pouch = Holder::Inside(id(1));
    // Four apples weigh more than a pouch takes of one thing, but each doesn't.
    inventory
        .add(
            stack(10, ItemKind::Apple, 4),
            Holder::Inventory,
            4,
            0,
            Rotation::None,
        )
        .unwrap();
    assert_eq!(
        inventory.check_move(id(10), pouch, 0, 0, Rotation::None),
        Ok(())
    );
    // Eleven are more than it takes in all.
    inventory
        .add(
            stack(20, ItemKind::Apple, 11),
            Holder::Inventory,
            5,
            0,
            Rotation::None,
        )
        .unwrap();
    assert_eq!(
        inventory.check_move(id(20), pouch, 0, 0, Rotation::None),
        Err(MoveRefused::TooHeavy)
    );
}

#[test]
fn wearing_a_stack_wears_only_its_top() {
    let mut inventory = Inventory::default();
    inventory
        .add(
            stack(1, ItemKind::Hat, 3),
            Holder::Inventory,
            0,
            0,
            Rotation::None,
        )
        .unwrap();
    assert!(inventory.wear(id(1)).is_ok());
    assert!(inventory.is_worn(id(3)));
    assert_eq!(ids(inventory.find(id(1)).unwrap()), [id(1), id(2)]);
    assert!(inventory.placed(id(1)).is_some());
}

#[test]
fn splitting_takes_the_top_in_order() {
    let mut coins = stack(1, ItemKind::Coin, 5);
    let top = coins.split_top(2);
    assert_eq!(ids(&top), [id(4), id(5)]);
    assert_eq!(ids(&coins), [id(1), id(2), id(3)]);
    assert_eq!(coins.take_top().map(|one| one.id), Some(id(3)));
    assert_eq!(coins.own_weight(), ItemKind::Coin.def().weight);
}

#[test]
fn secrets_are_hidden_all_the_way_up_a_stack() {
    let mut hats = stack(1, ItemKind::Hat, 2);
    hats.above[0].favour = Some(Favour::Cursed);
    hats.hide_secrets();
    assert_eq!(hats.top().favour, None);
}

#[test]
fn a_stack_with_something_known_to_differ_takes_no_more() {
    let limits = StackLimits::default();
    let mut hats = stack(1, ItemKind::Hat, 2);
    hats.above[0].favour = Some(Favour::Cursed);
    hats.above[0].known.favour = true;
    assert!(!hats.is_even());
    assert_eq!(limits.room(&hats, &Item::new(id(9), ItemKind::Hat)), 0);
    assert_eq!(
        StackLimits::NONE.room(
            &stack(10, ItemKind::Coin, 1),
            &Item::new(id(9), ItemKind::Coin)
        ),
        0
    );
}

#[test]
fn a_part_is_named_by_its_lowest_item_and_lifted_out_where_the_stack_lies() {
    let mut inventory = Inventory::default();
    inventory
        .add(
            stack(1, ItemKind::Coin, 5),
            Holder::Inventory,
            2,
            1,
            Rotation::None,
        )
        .unwrap();
    assert_eq!(inventory.part_of(id(1), 2), Some(id(4)));
    assert_eq!(inventory.part_of(id(1), 5), Some(id(1)));
    assert_eq!(inventory.part_of(id(1), 6), None);
    assert_eq!(inventory.lift_part(id(4)), Some(id(1)));
    let (_, part) = inventory.placed(id(4)).unwrap();
    assert_eq!((part.x, part.y, part.item.count()), (2, 1, 2));
    assert_eq!(inventory.find(id(1)).unwrap().count(), 3);
    // Nothing to lift for a whole stack or something alone.
    assert_eq!(inventory.lift_part(id(1)), None);
}

#[test]
fn a_lifted_part_moves_like_anything_else_or_goes_back() {
    let mut inventory = Inventory::default();
    inventory
        .add(
            stack(1, ItemKind::Coin, 5),
            Holder::Inventory,
            0,
            0,
            Rotation::None,
        )
        .unwrap();
    inventory.lift_part(id(4));
    // Where its stack is there is no room for it, so it can't stay.
    assert_eq!(
        inventory.move_item(id(4), Holder::Inventory, 0, 0, Rotation::None),
        Err(MoveRefused::NoRoom)
    );
    assert!(inventory.settle_parts());
    assert_eq!(
        ids(inventory.find(id(1)).unwrap()),
        (1..=5).map(id).collect::<Vec<_>>()
    );
    // Moved elsewhere, it stays a stack of its own.
    inventory.lift_part(id(4));
    inventory
        .move_item(id(4), Holder::Inventory, 3, 0, Rotation::None)
        .unwrap();
    assert!(!inventory.settle_parts());
    assert_eq!(ids(inventory.find(id(4)).unwrap()), [id(4), id(5)]);
    assert_eq!(inventory.find(id(1)).unwrap().count(), 3);
}

#[test]
fn wearing_a_lifted_part_wears_its_top_and_the_rest_goes_back() {
    let mut inventory = Inventory::default();
    inventory
        .add(
            stack(1, ItemKind::Hat, 4),
            Holder::Inventory,
            0,
            0,
            Rotation::None,
        )
        .unwrap();
    inventory.lift_part(id(3));
    inventory.wear(id(3)).unwrap();
    assert!(inventory.is_worn(id(4)));
    inventory.settle_parts();
    assert_eq!(ids(inventory.find(id(1)).unwrap()), [id(1), id(2), id(3)]);
}

/// Three hats, the middle one known to be cursed: a mixed stack.
fn mixed_hats() -> Item {
    let mut hats = stack(1, ItemKind::Hat, 3);
    hats.above[0].favour = Some(Favour::Cursed);
    hats.above[0].known.favour = true;
    hats
}

#[test]
fn a_mixed_stack_falls_into_groups_from_the_top() {
    let hats = mixed_hats();
    let groups: Vec<Vec<Id>> = hats
        .groups()
        .iter()
        .map(|group| group.iter().map(|one| one.id).collect())
        .collect();
    assert_eq!(groups, [vec![id(3), id(1)], vec![id(2)]]);
    assert_eq!(stack(5, ItemKind::Coin, 2).groups().len(), 1);
}

#[test]
fn a_group_lifts_out_keeping_both_orders_and_goes_back_if_not_moved() {
    let mut inventory = Inventory::default();
    inventory
        .add(mixed_hats(), Holder::Inventory, 0, 0, Rotation::None)
        .unwrap();
    // The plain ones, taking the bottom: what is left is known by its own lowest.
    assert_eq!(inventory.lift_group(id(1), &[id(1), id(3)]), Some(id(1)));
    assert_eq!(ids(inventory.find(id(1)).unwrap()), [id(1), id(3)]);
    assert_eq!(ids(inventory.find(id(2)).unwrap()), [id(2)]);
    assert!(inventory.settle_parts());
    assert_eq!(inventory.items.len(), 1);
    assert_eq!(inventory.find(id(2)).unwrap().count(), 3);
    // All of it, none of it or something else isn't a group.
    assert_eq!(inventory.lift_group(id(2), &[]), None);
    assert_eq!(inventory.lift_group(id(2), &[id(1), id(2), id(3)]), None);
    assert_eq!(inventory.lift_group(id(2), &[id(9)]), None);
}

#[test]
fn knowing_there_is_no_curse_looks_like_not_knowing() {
    let mut known = Item::new(id(1), ItemKind::Hat);
    known.known.favour = true;
    assert!(known.looks_like(&Item::new(id(2), ItemKind::Hat)));
}
