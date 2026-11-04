use shared::item::ItemKind;
use uuid::Uuid;

use super::*;

const LIMITS: StackLimits = StackLimits {
    grams_per_cell: 2000,
    most: 99,
};

fn id(n: u128) -> Id {
    Id(Uuid::from_u128(n))
}

/// These kinds in the inventory, in the first places they fit, ids from 1.
fn carrying(kinds: &[ItemKind]) -> Inventory {
    let mut inventory = Inventory::default();
    for (index, &kind) in kinds.iter().enumerate() {
        let item = Item::new(id(index as u128 + 1), kind);
        inventory.give(item, StackLimits::NONE).expect("room");
    }
    inventory
}

fn held(n: u128) -> Held {
    Held {
        id: id(n),
        rotation: Rotation::None,
        grip: (0, 0),
    }
}

#[test]
fn something_new_goes_by_cell_or_into_a_bag() {
    let inventory = carrying(&[ItemKind::Pouch]);
    let ring = Item::new(id(9), ItemKind::Ring);
    let aim = target_for_new(
        &inventory,
        &ring,
        held(9),
        Some((Holder::Inventory, 3, 2)),
        LIMITS,
    );
    assert_eq!(
        drop_move(&inventory, held(9), aim),
        Some((Holder::Inventory, 3, 2, Rotation::None))
    );
    let aim = target_for_new(
        &inventory,
        &ring,
        held(9),
        Some((Holder::Inventory, 1, 1)),
        LIMITS,
    );
    assert_eq!(
        drop_move(&inventory, held(9), aim),
        Some((Holder::Inside(id(1)), 0, 0, Rotation::None))
    );
}

#[test]
fn the_bin_isnt_a_move() {
    let inventory = carrying(&[ItemKind::Coin]);
    assert_eq!(drop_move(&inventory, held(1), Target::Bin), None);
}

#[test]
fn off_every_grid_it_snaps_back() {
    let inventory = carrying(&[ItemKind::Coin]);
    let aim = target(&inventory, held(1), None, LIMITS);
    assert_eq!(aim, Target::Nowhere);
    assert_eq!(drop_move(&inventory, held(1), aim), None);
}

#[test]
fn it_goes_where_the_gripped_cell_is_held() {
    let inventory = carrying(&[ItemKind::Stone]);
    let grip = Held {
        grip: (1, 1),
        ..held(1)
    };
    let aim = target(&inventory, grip, Some((Holder::Inventory, 3, 2)), LIMITS);
    assert_eq!(
        drop_move(&inventory, grip, aim),
        Some((Holder::Inventory, 2, 1, Rotation::None))
    );
}

#[test]
fn hanging_off_the_grid_or_on_something_else_it_snaps_back() {
    let inventory = carrying(&[ItemKind::Stone, ItemKind::Coin]);
    let grip = Held {
        grip: (1, 1),
        ..held(1)
    };
    for over in [(0, 0), (6, 3), (3, 1)] {
        let aim = target(
            &inventory,
            grip,
            Some((Holder::Inventory, over.0, over.1)),
            LIMITS,
        );
        let spot = matches!(aim, Target::Cells { allowed: false, .. });
        assert!(spot, "{over:?}: {aim:?}");
        assert_eq!(drop_move(&inventory, grip, aim), None);
    }
}

#[test]
fn dropped_back_where_it_was_nothing_needs_asking() {
    let inventory = carrying(&[ItemKind::Stone]);
    let aim = target(&inventory, held(1), Some((Holder::Inventory, 0, 0)), LIMITS);
    assert_eq!(drop_move(&inventory, held(1), aim), None);
}

#[test]
fn on_a_bag_it_goes_inside_if_it_may() {
    let inventory = carrying(&[ItemKind::Pouch, ItemKind::Ring, ItemKind::Stone]);
    let aim = target(&inventory, held(2), Some((Holder::Inventory, 1, 1)), LIMITS);
    assert_eq!(
        drop_move(&inventory, held(2), aim),
        Some((Holder::Inside(id(1)), 0, 0, Rotation::None))
    );
    // A stone is too big for a pouch, so it stays where it was.
    let aim = target(&inventory, held(3), Some((Holder::Inventory, 1, 1)), LIMITS);
    assert!(matches!(aim, Target::Bag { place: None, .. }));
    assert_eq!(drop_move(&inventory, held(3), aim), None);
}

#[test]
fn a_bag_held_over_its_own_place_is_not_dropped_in_itself() {
    let inventory = carrying(&[ItemKind::Pouch]);
    let aim = target(&inventory, held(1), Some((Holder::Inventory, 1, 1)), LIMITS);
    assert!(matches!(aim, Target::Cells { .. }));
}

#[test]
fn turning_keeps_hold_of_the_same_cell() {
    // A sword held by its second cell from the top.
    let inventory = carrying(&[ItemKind::Sword]);
    let sword = Held {
        grip: (0, 1),
        rotation: Rotation::Quarter,
        ..held(1)
    };
    // As it was given, the sword lies across: this grip is on a 5 × 1 shape.
    let across = Held {
        grip: (1, 0),
        ..sword
    };
    let turned = across.turned(&inventory);
    assert_eq!(turned.rotation, Rotation::Half);
    assert_eq!(turned.grip, (0, 1));
    // Four turns come back to where it started.
    let mut round = across;
    for _ in 0..4 {
        round = round.turned(&inventory);
    }
    assert_eq!(round, across);
}

#[test]
fn something_that_looks_the_same_goes_on_its_stack() {
    let mut inventory = Inventory::default();
    let apples = [
        Item::new(id(1), ItemKind::Apple),
        Item::new(id(2), ItemKind::Apple),
    ];
    for (x, apple) in apples.into_iter().enumerate() {
        inventory
            .add(apple, Holder::Inventory, x as u8, 0, Rotation::None)
            .unwrap();
    }
    let aim = target(&inventory, held(2), Some((Holder::Inventory, 0, 0)), LIMITS);
    assert_eq!(
        aim,
        Target::Stack {
            onto: id(1),
            allowed: true
        }
    );
    assert_eq!(stack_onto(aim), Some(id(1)));
    assert_eq!(drop_move(&inventory, held(2), aim), None);
    // A full stack shows red.
    let aim = target(
        &inventory,
        held(2),
        Some((Holder::Inventory, 0, 0)),
        StackLimits::NONE,
    );
    assert_eq!(stack_onto(aim), None);
    // Something different goes by cell as ever, and is refused.
    let coin = Item::new(id(3), ItemKind::Coin);
    let aim = target_for_new(
        &inventory,
        &coin,
        held(3),
        Some((Holder::Inventory, 0, 0)),
        LIMITS,
    );
    assert!(matches!(aim, Target::Cells { allowed: false, .. }));
}

#[test]
fn on_a_closed_bag_it_goes_on_a_stack_inside_first() {
    let mut inventory = carrying(&[ItemKind::Pouch]);
    let coin = Item::new(id(5), ItemKind::Coin);
    inventory
        .add(coin, Holder::Inside(id(1)), 3, 3, Rotation::None)
        .unwrap();
    let another = Item::new(id(6), ItemKind::Coin);
    let aim = target_for_new(
        &inventory,
        &another,
        held(6),
        Some((Holder::Inventory, 0, 0)),
        LIMITS,
    );
    assert_eq!(stack_onto(aim), Some(id(5)));
    assert_eq!(drop_move(&inventory, held(6), aim), None);
}
