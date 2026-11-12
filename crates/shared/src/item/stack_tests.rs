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
            .give(
                Item::new(id(n), ItemKind::Apple),
                limits,
                &mut crate::id::stand_ins(),
            )
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
            .give(
                Item::new(id(n), ItemKind::GoldCoin),
                limits,
                &mut crate::id::stand_ins(),
            )
            .unwrap();
    }
    let counts: Vec<u32> = inventory.items.iter().map(|p| p.item.count()).collect();
    assert_eq!(counts, [99, 1]);
}

#[test]
fn only_what_looks_the_same_stacks() {
    let apple = Item::new(id(1), ItemKind::Apple);
    assert!(apple.looks_like(&Item::new(id(2), ItemKind::Apple)));
    assert!(!apple.looks_like(&Item::new(id(3), ItemKind::CopperCoin)));
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
    assert_eq!(
        inventory.stack_onto(id(20), id(1), limits, &mut crate::id::stand_ins()),
        Ok(1)
    );
    // The top went first; what is left keeps its bottom, and so its id.
    let big = inventory.find(id(1)).unwrap();
    assert_eq!((big.count(), big.top().id), (13, id(22)));
    assert_eq!(ids(inventory.find(id(20)).unwrap()), [id(20), id(21)]);
    // Full now.
    assert_eq!(
        inventory.stack_onto(id(20), id(1), limits, &mut crate::id::stand_ins()),
        Err(MoveRefused::NoStackRoom)
    );
}

#[test]
fn a_whole_stack_joins_another_and_goes() {
    let limits = StackLimits::default();
    let mut inventory = Inventory::default();
    let apple = Item::new(id(3), ItemKind::Apple);
    inventory
        .add(
            stack(1, ItemKind::Apple, 2),
            Holder::Inventory,
            0,
            0,
            Rotation::None,
        )
        .unwrap();
    inventory
        .add(apple, Holder::Inventory, 2, 0, Rotation::None)
        .unwrap();
    assert_eq!(
        inventory.stack_onto(id(1), id(3), limits, &mut crate::id::stand_ins()),
        Ok(2)
    );
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
            stack(1, ItemKind::CopperCoin, 2),
            Holder::Inventory,
            0,
            0,
            Rotation::None,
        )
        .unwrap();
    let rest = inventory
        .add_to_stack(
            stack(10, ItemKind::CopperCoin, 3),
            id(1),
            limits,
            &mut crate::id::stand_ins(),
        )
        .unwrap();
    assert_eq!(ids(&rest.unwrap()), [id(10)]);
    assert_eq!(inventory.find(id(1)).unwrap().count(), 4);
    let apple = Item::new(id(20), ItemKind::Apple);
    assert_eq!(
        inventory.add_to_stack(apple, id(1), limits, &mut crate::id::stand_ins()),
        Err(MoveRefused::NoStackRoom)
    );
}

#[test]
fn a_bag_counts_one_item_of_a_stack_against_its_most_for_each() {
    let mut inventory = Inventory::default();
    inventory
        .give(
            Item::new(id(1), ItemKind::Pouch),
            StackLimits::NONE,
            &mut crate::id::stand_ins(),
        )
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
    let no_new_ids = &mut || unreachable!("apples aren't counted");
    let mut apples = stack(1, ItemKind::Apple, 5);
    let top = apples.split_top(2, no_new_ids);
    assert_eq!(ids(&top), [id(4), id(5)]);
    assert_eq!(ids(&apples), [id(1), id(2), id(3)]);
    assert_eq!(apples.take_top(no_new_ids).map(|one| one.id), Some(id(3)));
    assert_eq!(apples.own_weight(), ItemKind::Apple.def().weight);
}

#[test]
fn splitting_a_counted_stack_makes_a_new_one() {
    let mut coins = stack(1, ItemKind::CopperCoin, 5);
    assert_eq!((ids(&coins), coins.count()), (vec![id(1)], 5));
    let top = coins.split_top(2, &mut || id(9));
    assert_eq!((top.id, top.count(), coins.count()), (id(9), 2, 3));
    assert_eq!(
        coins.take_top(&mut || id(10)).map(|one| one.id),
        Some(id(10))
    );
    assert_eq!(coins.count(), 2);
    assert_eq!(coins.own_weight(), ItemKind::CopperCoin.def().weight);
    assert_eq!(coins.total_weight(), 2 * ItemKind::CopperCoin.def().weight);
    let mut one = stack(20, ItemKind::CopperCoin, 1);
    assert_eq!(
        one.take_top(&mut || id(11)),
        None,
        "alone, it is its own top"
    );
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
            &stack(10, ItemKind::CopperCoin, 1),
            &Item::new(id(9), ItemKind::CopperCoin)
        ),
        0
    );
}

#[test]
fn a_part_is_named_by_its_lowest_item_and_lifted_out_where_the_stack_lies() {
    let mut inventory = Inventory::default();
    inventory
        .add(
            stack(1, ItemKind::Apple, 5),
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
            stack(1, ItemKind::Apple, 5),
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
    assert_eq!(stack(5, ItemKind::CopperCoin, 2).groups().len(), 1);
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

#[test]
fn the_most_in_a_stack_goes_by_weight_and_count_and_bags_never_stack() {
    let limits = StackLimits::default();
    let coin = Item::new(id(1), ItemKind::CopperCoin);
    assert_eq!(limits.most_of(&coin), limits.most);
    let apple = Item::new(id(2), ItemKind::Apple);
    let room = limits.room(
        &stack(10, ItemKind::Apple, 1),
        &stack(20, ItemKind::Apple, 98),
    );
    assert_eq!(
        limits.most_of(&apple),
        room + 1,
        "as many as given ones stack to"
    );
    let bag = Item::new(id(3), ItemKind::Backpack);
    assert_eq!(limits.most_of(&bag), 1);
    assert_eq!(StackLimits::NONE.most_of(&coin), 1);
}

#[test]
fn part_of_a_counted_stack_is_lifted_out_with_a_new_id() {
    let mut inventory = Inventory::default();
    inventory
        .add(
            stack(1, ItemKind::CopperCoin, 5),
            Holder::Inventory,
            2,
            1,
            Rotation::None,
        )
        .unwrap();
    // No id names part of it until it is lifted.
    assert_eq!(inventory.part_of(id(1), 2), None);
    assert_eq!(inventory.part_of(id(1), 5), Some(id(1)));
    let mut new_id = || id(9);
    assert_eq!(inventory.lift_count(id(1), 6, &mut new_id), None);
    assert_eq!(inventory.lift_count(id(1), 5, &mut new_id), Some(id(1)));
    assert_eq!(inventory.lift_count(id(1), 2, &mut new_id), Some(id(9)));
    let (_, part) = inventory.placed(id(9)).unwrap();
    assert_eq!((part.x, part.y, part.item.count()), (2, 1, 2));
    assert_eq!(inventory.find(id(1)).unwrap().count(), 3);
    // Not moved, it goes back, and its id with it.
    assert!(inventory.settle_parts());
    assert_eq!(inventory.find(id(1)).unwrap().count(), 5);
    assert!(inventory.find(id(9)).is_none());
}

#[test]
fn counted_stacks_join_by_number_within_the_limits() {
    let limits = StackLimits {
        grams_per_cell: 2000,
        most: 10,
    };
    let mut inventory = Inventory::default();
    for (n, x) in [(1, 0), (2, 2)] {
        inventory
            .add(
                stack(n, ItemKind::GoldCoin, 7),
                Holder::Inventory,
                x,
                0,
                Rotation::None,
            )
            .unwrap();
    }
    let gold = inventory.find(id(1)).unwrap().clone();
    assert_eq!(limits.room(&gold, inventory.find(id(2)).unwrap()), 3);
    let stand_ins = &mut crate::id::stand_ins();
    assert_eq!(inventory.stack_onto(id(2), id(1), limits, stand_ins), Ok(3));
    assert_eq!(inventory.find(id(1)).unwrap().count(), 10);
    assert_eq!(inventory.find(id(2)).unwrap().count(), 4, "the rest stays");
    // By weight too: 5 g each, 2000 g for the cell.
    let heavy = StackLimits {
        grams_per_cell: 2000,
        most: 1000,
    };
    assert_eq!(heavy.most_of(&gold), 400);
    let full = Item {
        count: 398,
        ..gold.clone()
    };
    assert_eq!(heavy.room(&full, &stack(3, ItemKind::GoldCoin, 5)), 2);
    // Different counts still look the same.
    assert!(full.looks_like(&gold));
}

#[test]
fn old_saves_fold_stacked_coins_into_a_count() {
    let mut old = Item::new(id(1), ItemKind::CopperCoin);
    old.above = (2..=4)
        .map(|n| Item::new(id(n), ItemKind::CopperCoin))
        .collect();
    let text = ron::to_string(&old).unwrap();
    let loaded: Item = ron::from_str(&text).unwrap();
    assert_eq!((loaded.id, loaded.count, loaded.above.len()), (id(1), 4, 0));
    // Without a count written, it is 1; other kinds keep their stacks.
    let apples = stack(1, ItemKind::Apple, 3);
    let text = ron::to_string(&apples).unwrap().replace(",count:1", "");
    assert!(!text.contains("count"));
    let loaded: Item = ron::from_str(&text).unwrap();
    assert_eq!(loaded, apples);
}

#[test]
fn counted_kinds_are_plain() {
    // A stack of them is a number, so nothing may tell one from another.
    for kind in ItemKind::ALL {
        let def = kind.def();
        if !def.counted {
            continue;
        }
        assert!(!def.has_quality, "{kind:?} has quality");
        assert!(def.container.is_none(), "{kind:?} has a grid");
        assert!(def.wear.is_none(), "{kind:?} is worn");
        assert!(!kind.comes_in_sizes(), "{kind:?} comes in sizes");
        let mut item = Item::new(id(1), kind);
        crate::item_form::give_form(&mut item, None, &mut || id(2));
        assert!(item.form.is_none(), "{kind:?} has a form");
        assert!(item.name.is_none(), "{kind:?} has its own name");
        assert!(!item.can_be_magical(), "{kind:?} can be magical");
        item.roll_secrets();
        assert!(
            item.favour.is_none() && !item.is_magical(),
            "{kind:?} has secrets"
        );
    }
}
