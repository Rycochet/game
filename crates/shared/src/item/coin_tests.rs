use super::*;
use crate::id::Id;

use ItemKind::{CopperCoin, GoldCoin, SilverCoin};

fn id(n: u128) -> Id {
    Id::for_test(n)
}

/// `count` coins of this kind, ids from `first`, the first at the bottom.
fn coins(first: u128, kind: ItemKind, count: u128) -> Item {
    let mut bottom = Item::new(id(first), kind);
    for n in 1..count {
        bottom.put_on(Item::new(id(first + n), kind));
    }
    bottom
}

fn inventory(size: (u8, u8), lying: Vec<(Item, u8, u8)>) -> Inventory {
    let items = lying
        .into_iter()
        .map(|(item, x, y)| Placed {
            item,
            x,
            y,
            rotation: Rotation::None,
        })
        .collect();
    Inventory {
        size,
        items,
        ..Inventory::default()
    }
}

/// Each stack carried: its kind, count and place.
fn stacks(inventory: &Inventory) -> Vec<(ItemKind, u32, u8, u8)> {
    let mut all: Vec<_> = (inventory.items.iter())
        .map(|p| (p.item.kind, p.item.count(), p.x, p.y))
        .collect();
    all.sort_by_key(|&(_, _, x, y)| (y, x));
    all
}

#[test]
fn amounts_make_the_fewest_coins() {
    assert_eq!(
        coin_stacks(12_345, 99),
        [(GoldCoin, 1), (SilverCoin, 23), (CopperCoin, 45)]
    );
    assert_eq!(coin_stacks(0, 99), []);
    assert_eq!(coin_stacks(1_000_000, 99), [(GoldCoin, 99), (GoldCoin, 1)]);
    let mut n = 0;
    let made = coin_items(10_203, 99, None, &mut || {
        n += 1;
        id(n)
    });
    let counts: Vec<_> = made.iter().map(|s| (s.kind, s.count())).collect();
    assert_eq!(counts, [(GoldCoin, 1), (SilverCoin, 2), (CopperCoin, 3)]);
    assert_eq!(n, 6, "every coin has its own id");
}

#[test]
fn old_coins_load_as_copper() {
    let kind: ItemKind = ron::from_str("Coin").unwrap();
    assert_eq!(kind, CopperCoin);
    assert_eq!(CopperCoin.def().weight, 1);
    assert_eq!(SilverCoin.def().weight, 3);
    assert_eq!(GoldCoin.def().weight, 5);
}

#[test]
fn a_hundred_copper_turn_into_silver() {
    let limits = StackLimits::default();
    let mut carried = inventory(
        (6, 4),
        vec![
            (coins(1, CopperCoin, 99), 0, 0),
            (coins(200, CopperCoin, 10), 3, 0),
        ],
    );
    assert_eq!(carried.stack_onto(id(200), id(1), limits), Ok(10));
    // The silver goes in the first free place, which the ten left.
    assert_eq!(
        stacks(&carried),
        [(CopperCoin, 9, 0, 0), (SilverCoin, 1, 1, 0)]
    );
    assert_eq!(carried.items[0].item.id, id(1), "the stack keeps its id");
    // The new coin has the id of the first of the hundred it was made from:
    // the nine at the bottom stay.
    let silver = carried.find(id(10)).expect("the silver coin");
    assert_eq!(silver.kind, SilverCoin);
}

#[test]
fn turned_coins_join_a_stack_of_their_kind_and_carry_on() {
    let limits = StackLimits::default();
    let mut carried = inventory(
        (6, 4),
        vec![
            (coins(1, CopperCoin, 60), 0, 0),
            (coins(500, SilverCoin, 99), 4, 2),
            (coins(700, GoldCoin, 3), 5, 3),
        ],
    );
    let ground = coins(200, CopperCoin, 40);
    assert_eq!(carried.add_to_stack(ground, id(1), limits), Ok(None));
    // 100 copper make a silver, and 100 silver a gold, on the gold stack.
    assert_eq!(stacks(&carried), [(GoldCoin, 4, 5, 3)]);
}

#[test]
fn coins_that_cant_turn_stay_where_they_were() {
    let limits = StackLimits::default();
    // A one-cell inventory has room for a silver coin only where the copper
    // was: one of ten goes, and nine stay on the ground.
    let mut carried = inventory((1, 1), vec![(coins(1, CopperCoin, 99), 0, 0)]);
    let ground = coins(200, CopperCoin, 10);
    let rest = carried.add_to_stack(ground, id(1), limits).unwrap();
    assert_eq!(rest.map(|rest| rest.count()), Some(9));
    assert_eq!(stacks(&carried), [(SilverCoin, 1, 0, 0)]);
    // Silver with a full grid: none can go.
    let mut carried = inventory((1, 1), vec![(coins(1, SilverCoin, 99), 0, 0)]);
    carried.items[0].item.above.truncate(97);
    let ground = coins(200, SilverCoin, 3);
    let rest = carried.add_to_stack(ground, id(1), limits).unwrap();
    assert_eq!(rest.map(|rest| rest.count()), Some(1), "two make a gold");
    assert_eq!(stacks(&carried), [(GoldCoin, 1, 0, 0)]);
    let mut carried = inventory(
        (2, 1),
        vec![
            (coins(1, CopperCoin, 99), 0, 0),
            (coins(500, GoldCoin, 1), 1, 0),
        ],
    );
    let held = coins(200, CopperCoin, 3);
    // Two would leave copper where the silver must go; one leaves none.
    let rest = carried.add_to_stack(held, id(1), limits).unwrap();
    assert_eq!(rest.map(|rest| rest.count()), Some(2));
    // With all of them turned, the copper's own place takes the silver.
    let mut carried = inventory((1, 1), vec![(coins(1, CopperCoin, 60), 0, 0)]);
    let ground = coins(200, CopperCoin, 40);
    assert_eq!(carried.add_to_stack(ground, id(1), limits), Ok(None));
    assert_eq!(stacks(&carried), [(SilverCoin, 1, 0, 0)]);
}

#[test]
fn only_as_many_go_as_can_turn() {
    let limits = StackLimits::default();
    // No free place: a silver coin only has room where the copper was.
    let mut carried = inventory(
        (2, 1),
        vec![
            (coins(1, CopperCoin, 99), 0, 0),
            (coins(500, GoldCoin, 1), 1, 0),
        ],
    );
    // Of 199 more, only 101 can go: they make two silver where the copper was.
    // Any more or fewer would leave copper there, and the silver nowhere.
    let ground = coins(200, CopperCoin, 199);
    let rest = carried.add_to_stack(ground, id(1), limits).unwrap();
    assert_eq!(rest.map(|rest| rest.count()), Some(98));
    assert_eq!(
        stacks(&carried),
        [(SilverCoin, 2, 0, 0), (GoldCoin, 1, 1, 0)]
    );
}

#[test]
fn gold_stops_at_99() {
    let limits = StackLimits::default();
    let mut carried = inventory((6, 4), vec![(coins(1, GoldCoin, 99), 0, 0)]);
    let ground = coins(200, GoldCoin, 1);
    assert_eq!(
        carried.add_to_stack(ground, id(1), limits),
        Err(MoveRefused::NoStackRoom)
    );
}
