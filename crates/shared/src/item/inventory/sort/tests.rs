use super::*;
use crate::item::{ItemKind, Rotation};

fn id(n: u128) -> Id {
    Id::for_test(n)
}

fn at(n: u128, kind: ItemKind, x: u8, y: u8, rotation: Rotation) -> Placed {
    Placed {
        item: Item::new(id(n), kind),
        x,
        y,
        rotation,
    }
}

/// A six by four inventory, with a sword along the top.
fn messy() -> Inventory {
    Inventory {
        items: vec![
            at(1, ItemKind::Apple, 0, 3, Rotation::None),
            at(2, ItemKind::CopperCoin, 5, 0, Rotation::None),
            at(3, ItemKind::Sword, 0, 0, Rotation::Quarter),
            at(4, ItemKind::Apple, 5, 3, Rotation::None),
            at(5, ItemKind::Stone, 2, 1, Rotation::None),
        ],
        ..Inventory::default()
    }
}

/// Each item's id, place and how many it is.
fn layout(grid: &[Placed]) -> Vec<(Id, u8, u8, Rotation, u32)> {
    let each = grid.iter().map(|placed| {
        let item = &placed.item;
        (item.id, placed.x, placed.y, placed.rotation, item.count())
    });
    each.collect()
}

#[test]
fn largest_go_first_from_the_top_left_with_like_stacked() {
    let sorted = messy()
        .sorted_grid(Holder::Inventory, StackLimits::default())
        .unwrap();
    assert_eq!(
        layout(&sorted),
        [
            // The sword keeps its turn, along the top.
            (id(3), 0, 0, Rotation::Quarter, 1),
            (id(5), 0, 1, Rotation::None, 1),
            // The apples are one stack, the first in at the bottom; food before
            // valuables among things the same size.
            (id(1), 5, 0, Rotation::None, 2),
            (id(2), 2, 1, Rotation::None, 1),
        ]
    );
    assert_eq!(sorted[2].item.top().id, id(4));
}

#[test]
fn something_turns_only_if_it_must() {
    let mut inventory = messy();
    // Upright, it is too tall for the inventory.
    inventory.items[2].rotation = Rotation::None;
    inventory.items[2].y = 1;
    let sorted = inventory
        .sorted_grid(Holder::Inventory, StackLimits::default())
        .unwrap();
    assert_eq!((sorted[0].item.id, sorted[0].x, sorted[0].y), (id(3), 0, 0));
    assert_eq!(sorted[0].rotation, Rotation::Quarter);
}

#[test]
fn stacks_keep_to_their_limits() {
    let mut inventory = messy();
    let limits = StackLimits {
        most: 1,
        ..StackLimits::default()
    };
    assert!(inventory.sort_grid(Holder::Inventory, limits));
    assert_eq!(inventory.items.len(), 5);
    assert!(
        inventory
            .items
            .iter()
            .all(|placed| placed.item.count() == 1)
    );
}

#[test]
fn sorting_again_changes_nothing() {
    let mut inventory = messy();
    let limits = StackLimits::default();
    assert!(inventory.sort_grid(Holder::Inventory, limits));
    let once = inventory.clone();
    assert!(!inventory.sort_grid(Holder::Inventory, limits));
    assert_eq!(inventory, once);
}

#[test]
fn nothing_changes_if_something_would_not_fit() {
    // Two things that can't stack, in an inventory of one cell.
    let mut inventory = Inventory {
        items: vec![
            at(1, ItemKind::Apple, 0, 0, Rotation::None),
            at(2, ItemKind::CopperCoin, 0, 0, Rotation::None),
        ],
        size: (1, 1),
        ..Inventory::default()
    };
    let before = inventory.clone();
    assert_eq!(
        inventory.sorted_grid(Holder::Inventory, StackLimits::default()),
        None
    );
    assert!(!inventory.sort_grid(Holder::Inventory, StackLimits::default()));
    assert_eq!(inventory, before);
}

#[test]
fn a_bag_inside_is_sorted_on_its_own() {
    let mut backpack = Item::new(id(10), ItemKind::Backpack);
    backpack.contents = vec![
        at(11, ItemKind::CopperCoin, 3, 3, Rotation::None),
        at(12, ItemKind::CopperCoin, 1, 2, Rotation::None),
    ];
    let mut inventory = Inventory {
        items: vec![Placed {
            item: backpack,
            x: 0,
            y: 0,
            rotation: Rotation::None,
        }],
        ..Inventory::default()
    };
    let carried = inventory.items.clone();
    assert!(inventory.sort_grid(Holder::Inside(id(10)), StackLimits::default()));
    let inside = inventory.grid(Holder::Inside(id(10))).unwrap();
    assert_eq!(layout(inside), [(id(11), 0, 0, Rotation::None, 2)]);
    assert_eq!(inventory.items[0].x, carried[0].x);
}
