use super::*;
use crate::creature::{CreatureKind, Size};
use crate::id::Id;
use crate::rng::Seed;

#[test]
fn the_odds_add_up_and_better_is_rarer_past_common() {
    let total: u64 = Quality::ALL.iter().map(|quality| quality.odds()).sum();
    assert_eq!(total, ODDS_OUT_OF);
    for pair in Quality::ALL[1..].windows(2) {
        assert!(pair[0].odds() > pair[1].odds(), "{pair:?}");
    }
}

#[test]
fn rolls_of_quality_follow_the_odds() {
    let mut rng = Seed::new(12).rng();
    let mut counts = [0u64; 7];
    for _ in 0..100_000 {
        counts[Quality::roll(&mut rng) as usize] += 1;
    }
    for quality in Quality::ALL {
        let expected = quality.odds() * 10;
        let got = counts[quality as usize];
        assert!(
            got.abs_diff(expected) <= expected / 4 + 10,
            "{quality:?}: {got}"
        );
    }
}

#[test]
fn the_same_seed_rolls_the_same_quality() {
    let rolls = |seed| {
        let mut rng = Seed::new(seed).rng();
        [(); 16].map(|_| Quality::roll(&mut rng))
    };
    assert_eq!(rolls(1), rolls(1));
    assert_ne!(rolls(1), rolls(2));
}

#[test]
fn a_drop_takes_the_nearest_quality_the_table_has() {
    let line = |quality, kind| Drop {
        quality,
        kind,
        weight: 1,
    };
    assert_eq!(roll_drop(&mut Seed::new(1).rng(), &[]), None);
    // Only Rare in the table, whatever is rolled.
    let table = [line(Quality::Rare, ItemKind::Sword)];
    for seed in 0..200 {
        let dropped = roll_drop(&mut Seed::new(seed).rng(), &table);
        assert_eq!(dropped, Some((ItemKind::Sword, Some(Quality::Rare))));
    }
    // Each roll stays within what is at or below it, once any is.
    let table = [
        line(Quality::Poor, ItemKind::Hat),
        line(Quality::Epic, ItemKind::Sword),
    ];
    let mut swords = 0;
    for seed in 0..2000 {
        match roll_drop(&mut Seed::new(seed).rng(), &table) {
            Some((ItemKind::Hat, Some(Quality::Poor))) => {}
            Some((ItemKind::Sword, Some(Quality::Epic))) => swords += 1,
            other => panic!("{other:?}"),
        }
    }
    assert!((1..100).contains(&swords), "{swords}");
}

#[test]
fn an_item_goes_by_its_own_name_if_it_has_one() {
    let mut sword = Item::new(id(1), ItemKind::Sword);
    assert_eq!(sword.name(), "Sword");
    sword.name = Some(artifact_name(&mut Seed::new(1).rng(), ItemKind::Sword));
    assert!(sword.name().starts_with("The "), "{}", sword.name());
    assert!(sword.name().ends_with(" Sword"), "{}", sword.name());
}

#[test]
fn a_kind_without_quality_drops_without_one() {
    let table = [Drop {
        quality: Quality::Common,
        kind: ItemKind::CopperCoin,
        weight: 1,
    }];
    let dropped = roll_drop(&mut Seed::new(3).rng(), &table);
    assert_eq!(dropped, Some((ItemKind::CopperCoin, None)));
    assert!(ItemKind::Sword.def().has_quality);
}

/// An inventory with these kinds in it, ids from 1, left to right.
fn carrying(kinds: &[ItemKind]) -> Inventory {
    let mut inventory = Inventory::default();
    for (index, &kind) in kinds.iter().enumerate() {
        let item = Item::new(id(index as u128 + 1), kind);
        inventory
            .give(item, StackLimits::NONE, &mut crate::id::stand_ins())
            .expect("room in the inventory");
    }
    inventory
}

fn id(n: u128) -> Id {
    Id::for_test(n)
}

const NONE: Rotation = Rotation::None;

#[test]
fn every_kind_has_a_sensible_shape_and_is_listed_alphabetically() {
    for kind in ItemKind::ALL {
        let def = kind.def();
        assert!(def.shape.size() > 0, "{}", def.name);
        assert!(def.shape.width <= MAX_SHAPE_SIDE && def.shape.height <= MAX_SHAPE_SIDE);
        assert!(def.weight > 0);
    }
    let names: Vec<_> = ItemKind::ALL.iter().map(|kind| kind.def().name).collect();
    let mut sorted = names.clone();
    sorted.sort();
    assert_eq!(names, sorted);
}

fn sized(n: u128, kind: ItemKind, size: Size) -> Item {
    Item {
        made: Made::for_body(kind, size, Some(10), false),
        ..Item::new(id(n), kind)
    }
}

fn sides(kind: ItemKind, size: Size) -> (u8, u8) {
    let shape = sized(1, kind, size).shape();
    (shape.width, shape.height)
}

#[test]
fn sizes_round_down_going_up_and_up_going_down() {
    let ring = |size| sides(ItemKind::Ring, size);
    for size in [Size::Tiny, Size::Small, Size::Medium, Size::Large] {
        assert_eq!(ring(size), (1, 1), "{size:?}");
    }
    assert_eq!(ring(Size::Huge), (2, 2));
    assert_eq!(ring(Size::Giant), (3, 3));

    let sword = |size| sides(ItemKind::Sword, size);
    assert_eq!(sword(Size::Tiny), (1, 2));
    assert_eq!(sword(Size::Small), (1, 4));
    assert_eq!(sword(Size::Medium), (1, 5));
    assert_eq!(sword(Size::Large), (1, 7));
}

#[test]
fn no_side_passes_the_most_a_shape_can_hold() {
    assert_eq!(sides(ItemKind::Sword, Size::Huge), (2, MAX_SHAPE_SIDE));
    assert_eq!(sides(ItemKind::Sword, Size::Giant), (3, MAX_SHAPE_SIDE));
}

#[test]
fn every_kind_keeps_its_cells_and_its_medium_shape_at_every_size() {
    for kind in ItemKind::ALL {
        assert_eq!(kind.def().shape.scaled(Size::Medium), kind.def().shape);
        for size in Size::ALL {
            let shape = kind.def().shape.scaled(size);
            assert!(shape.size() > 0, "{kind:?} {size:?}");
            assert!(shape.width <= MAX_SHAPE_SIDE && shape.height <= MAX_SHAPE_SIDE);
        }
    }
}

#[test]
fn a_gap_in_a_shape_grows_with_it() {
    let shape = ItemKind::Boomerang.def().shape.scaled(Size::Huge);
    assert_eq!((shape.width, shape.height), (4, 4));
    assert_eq!(shape.size(), 12);
    assert!(!shape.has(2, 0) && !shape.has(3, 1));
    assert!(shape.has(0, 0) && shape.has(3, 3));
}

#[test]
fn a_name_says_the_size_unless_it_is_medium_or_its_own() {
    assert_eq!(sized(1, ItemKind::Ring, Size::Medium).name(), "Ring");
    assert_eq!(sized(1, ItemKind::Ring, Size::Giant).name(), "Giant Ring");
    let named = Item {
        name: Some("The Band".to_owned()),
        ..sized(1, ItemKind::Ring, Size::Giant)
    };
    assert_eq!(named.name(), "The Band");
}

#[test]
fn a_name_says_what_is_known_of_blessings_curses_and_magic() {
    let ring = Item {
        made: Made {
            magical: true,
            ..sized(1, ItemKind::Ring, Size::Giant).made
        },
        quality: Some(Quality::Rare),
        favour: Some(Favour::Cursed),
        ..sized(1, ItemKind::Ring, Size::Giant)
    };
    assert_eq!(ring.name(), "Cursed Magical Giant Ring");
    let named = Item {
        name: Some("The Band".to_owned()),
        ..ring.clone()
    };
    assert_eq!(named.name(), "The Band (Cursed, Magical)");
    let mut hidden = ring.clone();
    hidden.known.favour = true;
    hidden.hide_secrets();
    assert_eq!(hidden.name(), "Cursed Giant Ring");
    let blessed = Item {
        favour: Some(Favour::Blessed),
        ..sized(1, ItemKind::Ring, Size::Medium)
    };
    assert_eq!(blessed.name(), "Blessed Ring");
}

#[test]
fn the_rules_go_by_the_size_it_was_made() {
    let mut inventory = carrying(&[ItemKind::Pouch]);
    let giant = sized(2, ItemKind::Ring, Size::Giant);
    // As big as the pouch's outside, so it can't go inside.
    assert_eq!(
        inventory.check_add(&giant, Holder::Inside(id(1)), 0, 0, NONE),
        Err(MoveRefused::TooBig)
    );
    // Beside the pouch, in the inventory, its second row and column count.
    assert_eq!(
        inventory.check_add(&giant, Holder::Inventory, 5, 0, NONE),
        Err(MoveRefused::NoRoom)
    );
    inventory.add(giant, Holder::Inventory, 2, 0, NONE).unwrap();
    assert_eq!(
        inventory.check_add(
            &Item::new(id(3), ItemKind::Ring),
            Holder::Inventory,
            3,
            1,
            NONE
        ),
        Err(MoveRefused::NoRoom)
    );
}

#[test]
fn things_saved_smaller_than_they_now_are_can_still_be_moved_apart() {
    // Saved at one cell each before sizes changed shapes: now they overlap.
    let mut inventory = Inventory::default();
    for (n, x) in [(1, 0), (2, 1)] {
        inventory.items.push(Placed {
            item: sized(n, ItemKind::Ring, Size::Giant),
            x,
            y: 0,
            rotation: NONE,
        });
    }
    inventory
        .move_item(id(2), Holder::Inventory, 3, 0, NONE)
        .unwrap();
    assert_eq!(inventory.items.len(), 2);
    assert!(inventory.find(id(1)).is_some() && inventory.find(id(2)).is_some());
}

#[test]
fn a_quarter_turn_swaps_the_sides_and_four_come_back() {
    let shape = ItemKind::Boomerang.def().shape;
    // An L: everything but the top right.
    assert!(shape.has(0, 0) && !shape.has(1, 0) && shape.has(0, 1) && shape.has(1, 1));
    let turned = shape.turned(Rotation::Quarter);
    // Clockwise, the missing corner goes to the bottom right.
    assert!(turned.has(0, 0) && turned.has(1, 0) && turned.has(0, 1) && !turned.has(1, 1));

    let sword = ItemKind::Sword.def().shape.turned(Rotation::Quarter);
    assert_eq!((sword.width, sword.height), (5, 1));
    let mut all_round = ItemKind::Boomerang.def().shape;
    for _ in 0..4 {
        all_round = all_round.turned(Rotation::Quarter);
    }
    assert_eq!(all_round, ItemKind::Boomerang.def().shape);
}

#[test]
fn things_cant_lie_on_each_other_or_stick_out() {
    let mut inventory = carrying(&[ItemKind::Stone, ItemKind::Apple]);
    assert_eq!(
        inventory.check_move(id(2), Holder::Inventory, 1, 1, NONE),
        Err(MoveRefused::NoRoom)
    );
    assert_eq!(
        inventory.check_move(id(1), Holder::Inventory, 5, 0, NONE),
        Err(MoveRefused::NoRoom)
    );
    // An item isn't in its own way.
    assert_eq!(
        inventory.check_move(id(1), Holder::Inventory, 1, 0, NONE),
        Err(MoveRefused::NoRoom),
        "the apple is at (2, 0)"
    );
    assert_eq!(
        inventory.move_item(id(2), Holder::Inventory, 5, 3, NONE),
        Ok(())
    );
    assert_eq!(
        inventory.move_item(id(1), Holder::Inventory, 1, 0, NONE),
        Ok(())
    );
    assert_eq!(inventory.placed(id(1)).unwrap().1.x, 1);
}

#[test]
fn a_shape_with_a_gap_leaves_it_free() {
    let inventory = carrying(&[ItemKind::Boomerang, ItemKind::CopperCoin]);
    // In the boomerang's missing corner.
    assert_eq!(inventory.placed(id(2)).unwrap().1.x, 1);
}

#[test]
fn only_a_smaller_shape_goes_in() {
    let inventory = carrying(&[ItemKind::Pouch, ItemKind::Stone]);
    // A 2 × 2 stone in a 2 × 2 pouch: the same size isn't smaller.
    assert_eq!(
        inventory.check_move(id(2), Holder::Inside(id(1)), 0, 0, NONE),
        Err(MoveRefused::TooBig)
    );
    // The outer shape matters, not the bigger grid inside.
    let sword = Item::new(id(9), ItemKind::Sword);
    let mut with_sword = carrying(&[ItemKind::Pouch]);
    with_sword
        .give(sword, StackLimits::NONE, &mut crate::id::stand_ins())
        .unwrap();
    assert_eq!(
        with_sword.check_move(id(9), Holder::Inside(id(1)), 0, 0, NONE),
        Err(MoveRefused::TooBig)
    );
}

#[test]
fn a_pouch_holds_more_than_its_outside() {
    let mut inventory = carrying(&[ItemKind::Pouch]);
    for ring in 0..16u8 {
        let ring_id = id(100 + ring as u128);
        inventory
            .give(
                Item::new(ring_id, ItemKind::Ring),
                StackLimits::NONE,
                &mut crate::id::stand_ins(),
            )
            .unwrap_or_else(|_| panic!("the inventory is full"));
        inventory
            .move_item(ring_id, Holder::Inside(id(1)), ring % 4, ring / 4, NONE)
            .unwrap();
    }
    assert_eq!(inventory.find(id(1)).unwrap().contents.len(), 16);
    assert_eq!(inventory.items.len(), 1);
    assert_eq!(inventory.total_weight(), 80 + 16 * 10);
}

#[test]
fn the_creature_counts_with_what_it_carries_and_wears_but_not_the_bin() {
    let mut inventory = carrying(&[ItemKind::Pouch, ItemKind::Ring, ItemKind::Ring]);
    inventory.creature = CreatureKind::Horse;
    assert_eq!(inventory.weight_with_creature(), 500_000 + 80 + 10 + 10);
    inventory
        .move_item(id(2), Holder::Inside(id(1)), 0, 0, NONE)
        .unwrap();
    assert!(inventory.to_bin(id(3)));
    inventory.worn.push(Item::new(id(4), ItemKind::Stone));
    let stone = ItemKind::Stone.def().weight;
    assert_eq!(inventory.weight_with_creature(), 500_000 + 80 + 10 + stone);
}

#[test]
fn nothing_goes_inside_itself() {
    let mut inventory = carrying(&[ItemKind::Backpack, ItemKind::Pouch]);
    inventory
        .move_item(id(2), Holder::Inside(id(1)), 0, 0, NONE)
        .unwrap();
    assert_eq!(
        inventory.check_move(id(1), Holder::Inside(id(1)), 0, 0, NONE),
        Err(MoveRefused::IntoItself)
    );
    assert_eq!(
        inventory.check_move(id(1), Holder::Inside(id(2)), 0, 0, NONE),
        Err(MoveRefused::IntoItself)
    );
}

/// A backpack holding a pouch holding a box, in the inventory, ids 1 to 3.
fn nested() -> Inventory {
    let mut inventory = carrying(&[ItemKind::Backpack, ItemKind::Pouch, ItemKind::SmallBox]);
    inventory
        .move_item(id(3), Holder::Inside(id(2)), 0, 0, NONE)
        .unwrap();
    inventory
        .move_item(id(2), Holder::Inside(id(1)), 0, 0, NONE)
        .unwrap();
    inventory
}

#[test]
fn three_grids_deep_is_the_limit() {
    // The backpack, pouch and box are three grids, and a ring may go in the last.
    let mut inventory = nested();
    inventory
        .give(
            Item::new(id(4), ItemKind::Ring),
            StackLimits::NONE,
            &mut crate::id::stand_ins(),
        )
        .unwrap();
    assert_eq!(
        inventory.check_move(id(4), Holder::Inside(id(3)), 0, 0, NONE),
        Ok(())
    );

    // Nothing small enough has a grid today, so a box holding a pouch is made
    // by hand: going into the pouch, its grids would be third and fourth.
    inventory
        .give(
            Item::new(id(5), ItemKind::SmallBox),
            StackLimits::NONE,
            &mut crate::id::stand_ins(),
        )
        .unwrap();
    let (_, placed) = inventory.placed(id(5)).unwrap();
    let (x, y) = (placed.x, placed.y);
    let index = inventory
        .items
        .iter()
        .position(|placed| placed.x == x && placed.y == y);
    inventory.items[index.unwrap()].item.contents.push(Placed {
        item: Item::new(id(6), ItemKind::Pouch),
        x: 0,
        y: 0,
        rotation: NONE,
    });
    assert_eq!(
        inventory.check_move(id(5), Holder::Inside(id(2)), 0, 1, NONE),
        Err(MoveRefused::TooDeep)
    );
    // One grid further out, it fits.
    assert_eq!(
        inventory.check_move(id(5), Holder::Inside(id(1)), 5, 0, NONE),
        Ok(())
    );
}

#[test]
fn a_container_limits_the_weight_of_each_thing_and_of_all() {
    // A lantern is 800 g, and a pouch takes 500 g each.
    let inventory = carrying(&[ItemKind::Pouch, ItemKind::Lantern]);
    assert_eq!(
        inventory.check_move(id(2), Holder::Inside(id(1)), 0, 0, NONE),
        Err(MoveRefused::TooHeavy)
    );
    // Ten apples of 150 g are the 1500 g it takes in all.
    let mut inventory = carrying(&[ItemKind::Pouch]);
    for apple in 0..11u8 {
        let apple_id = id(50 + apple as u128);
        inventory
            .give(
                Item::new(apple_id, ItemKind::Apple),
                StackLimits::NONE,
                &mut crate::id::stand_ins(),
            )
            .unwrap();
        let result =
            inventory.move_item(apple_id, Holder::Inside(id(1)), apple % 4, apple / 4, NONE);
        let wanted = if apple < 10 {
            Ok(())
        } else {
            Err(MoveRefused::TooHeavy)
        };
        assert_eq!(result, wanted);
    }
}

#[test]
fn weight_limits_look_at_the_containers_further_out() {
    // A box in a pouch, with nine apples beside it: 1450 g of the pouch's
    // 1500 g. Another apple is fine for the box, but not for the pouch.
    let mut inventory = carrying(&[ItemKind::Pouch, ItemKind::SmallBox]);
    inventory
        .move_item(id(2), Holder::Inside(id(1)), 0, 0, NONE)
        .unwrap();
    for apple in 0..9u8 {
        let apple_id = id(50 + apple as u128);
        inventory
            .give(
                Item::new(apple_id, ItemKind::Apple),
                StackLimits::NONE,
                &mut crate::id::stand_ins(),
            )
            .unwrap();
        inventory
            .move_item(
                apple_id,
                Holder::Inside(id(1)),
                apple % 4,
                1 + apple / 4,
                NONE,
            )
            .unwrap();
    }
    let last = id(70);
    inventory
        .give(
            Item::new(last, ItemKind::Apple),
            StackLimits::NONE,
            &mut crate::id::stand_ins(),
        )
        .unwrap();
    assert_eq!(
        inventory.check_move(last, Holder::Inside(id(2)), 0, 0, NONE),
        Err(MoveRefused::TooHeavy)
    );

    // A pouch made by hand inside a pouch can't grow past 500 g in the outer.
    let mut inventory = carrying(&[ItemKind::Pouch, ItemKind::Apple]);
    let mut inner = Item::new(id(3), ItemKind::Pouch);
    for apple in 0..3 {
        inner.contents.push(Placed {
            item: Item::new(id(10 + apple as u128), ItemKind::Apple),
            x: apple,
            y: 0,
            rotation: NONE,
        });
    }
    inventory.items[0].item.contents.push(Placed {
        item: inner,
        x: 0,
        y: 0,
        rotation: NONE,
    });
    assert_eq!(
        inventory.check_move(id(2), Holder::Inside(id(3)), 3, 0, NONE),
        Err(MoveRefused::TooHeavy)
    );
}

#[test]
fn moving_about_inside_a_bag_doesnt_count_its_weight_twice() {
    let mut inventory = carrying(&[ItemKind::Pouch]);
    for apple in 0..10u8 {
        let apple_id = id(50 + apple as u128);
        inventory
            .give(
                Item::new(apple_id, ItemKind::Apple),
                StackLimits::NONE,
                &mut crate::id::stand_ins(),
            )
            .unwrap();
        inventory
            .move_item(apple_id, Holder::Inside(id(1)), apple % 4, apple / 4, NONE)
            .unwrap();
    }
    // It holds all it can, and an apple can still move to a free place.
    assert_eq!(
        inventory.move_item(id(50), Holder::Inside(id(1)), 3, 3, NONE),
        Ok(())
    );
}

#[test]
fn dropped_on_a_closed_bag_it_takes_the_first_place_turning_if_it_must() {
    let inventory = carrying(&[ItemKind::Backpack, ItemKind::Lantern]);
    let place = inventory.first_place_for(id(2), Holder::Inside(id(1)), NONE);
    assert_eq!(place, Some((0, 0, NONE)));

    // A box 2 × 1 in a pouch filled but for the last column.
    let mut pouch = carrying(&[ItemKind::Pouch]);
    for y in 0..4u8 {
        for x in 0..3u8 {
            let coin = id(100 + (y as u128) * 10 + x as u128);
            pouch
                .give(
                    Item::new(coin, ItemKind::CopperCoin),
                    StackLimits::NONE,
                    &mut crate::id::stand_ins(),
                )
                .unwrap();
            pouch
                .move_item(coin, Holder::Inside(id(1)), x, y, NONE)
                .unwrap();
        }
    }
    pouch
        .give(
            Item::new(id(2), ItemKind::SmallBox),
            StackLimits::NONE,
            &mut crate::id::stand_ins(),
        )
        .unwrap();
    let place = pouch.first_place_for(id(2), Holder::Inside(id(1)), NONE);
    assert_eq!(place, Some((3, 0, Rotation::Quarter)));

    // Not a place at all when it isn't allowed in.
    assert_eq!(
        inventory.first_place_for(id(1), Holder::Inside(id(1)), NONE),
        None
    );
}

#[test]
fn the_bin_keeps_the_last_thing_thrown_away_until_the_next() {
    let mut inventory = nested();
    inventory
        .give(
            Item::new(id(4), ItemKind::CopperCoin),
            StackLimits::NONE,
            &mut crate::id::stand_ins(),
        )
        .unwrap();
    assert!(inventory.to_bin(id(2)));
    // Gone from the backpack, but it and what is in it can still be had.
    assert!(inventory.find(id(1)).unwrap().contents.is_empty());
    assert!(inventory.find(id(2)).is_some());
    assert!(inventory.placed(id(2)).is_none());
    // It weighs nothing there.
    assert_eq!(inventory.total_weight(), 1200 + 1);
    // Thrown away twice, nothing changes.
    assert!(!inventory.to_bin(id(2)));
    // Taken back, it goes in a grid again.
    assert_eq!(
        inventory.move_item(id(2), Holder::Inventory, 4, 0, NONE),
        Ok(())
    );
    assert!(inventory.bin.is_none());
    // Another thrown away destroys the first.
    assert!(inventory.to_bin(id(2)));
    assert!(inventory.to_bin(id(4)));
    assert!(inventory.find(id(2)).is_none());
    assert!(inventory.find(id(3)).is_none());
    assert!(inventory.find(id(4)).is_some());
}

#[test]
fn a_new_item_goes_in_the_first_place_or_comes_back() {
    let mut inventory = Inventory::default();
    for n in 0..24 {
        inventory
            .give(
                Item::new(id(n), ItemKind::CopperCoin),
                StackLimits::NONE,
                &mut crate::id::stand_ins(),
            )
            .unwrap();
    }
    let refused = inventory.give(
        Item::new(id(99), ItemKind::CopperCoin),
        StackLimits::NONE,
        &mut crate::id::stand_ins(),
    );
    assert_eq!(refused.unwrap_err().id, id(99));
}

#[test]
fn an_inventory_reads_back_from_a_message() {
    let inventory = nested();
    let bytes = postcard::to_allocvec(&inventory).unwrap();
    assert_eq!(
        postcard::from_bytes::<Inventory>(&bytes).unwrap(),
        inventory
    );
}

#[test]
fn a_bag_of_holding_takes_things_down_to_its_most_rows() {
    let container = ItemKind::BagOfHolding.def().container.unwrap();
    assert!(container.grows && !ItemKind::Backpack.def().container.unwrap().grows);
    let mut inventory = carrying(&[ItemKind::BagOfHolding]);
    let bag = Holder::Inside(id(1));
    let last_row = container.height - 1;
    let apple = Item::new(id(2), ItemKind::Apple);
    inventory.check_add(&apple, bag, 0, last_row, NONE).unwrap();
    assert!(
        inventory
            .check_add(&apple, bag, 0, container.height, NONE)
            .is_err()
    );
    // Its rules are those of a grid as big as it can grow.
    inventory
        .add(apple, bag, container.width - 1, 50, NONE)
        .unwrap();
    assert_eq!(
        inventory.grid_size(bag),
        Some((container.width, container.height))
    );
}
