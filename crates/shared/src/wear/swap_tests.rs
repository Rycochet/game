use super::*;
use crate::creature::CreatureKind;
use crate::id::Id;
use crate::item::{Favour, Inventory, Item, ItemKind, Made, StackLimits};

fn id(n: u128) -> Id {
    Id::for_test(n)
}

fn made(kind: ItemKind, n: u128) -> Item {
    let def = CreatureKind::Human.def();
    Item {
        made: Made::for_body(kind, def.size, def.fingers(), false),
        ..Item::new(id(n), kind)
    }
}

/// A person carrying these, ids from 1, in a big inventory.
fn carrying(kinds: &[ItemKind]) -> Inventory {
    let mut inventory = Inventory {
        creature: CreatureKind::Human,
        size: (40, 40),
        ..Inventory::default()
    };
    for (index, &kind) in kinds.iter().enumerate() {
        let item = made(kind, index as u128 + 1);
        inventory.give(item, StackLimits::NONE).expect("room");
    }
    inventory
}

fn swap(inventory: &mut Inventory, n: u128) -> Result<Swapped, WearRefused> {
    inventory.wear_swapping(id(n), false, StackLimits::NONE)
}

fn carried(inventory: &Inventory, n: u128) -> bool {
    inventory.items.iter().any(|placed| placed.item.id == id(n))
}

#[test]
fn what_is_in_the_way_comes_off_into_the_inventory() {
    let mut inventory = carrying(&[ItemKind::Hat, ItemKind::Hat]);
    inventory.wear(id(1)).unwrap();
    let swapped = swap(&mut inventory, 2).unwrap();
    assert_eq!(swapped.taken_off, [id(1)]);
    assert!(inventory.is_worn(id(2)));
    assert!(carried(&inventory, 1));
}

#[test]
fn one_place_takes_over_from_something_on_several() {
    // Gloves are on hands and fingers; fingerless ones only on hands.
    let mut inventory = carrying(&[ItemKind::Gloves, ItemKind::FingerlessGloves]);
    inventory.wear(id(1)).unwrap();
    assert_eq!(swap(&mut inventory, 2).unwrap().taken_off, [id(1)]);
    assert!(inventory.is_worn(id(2)) && !inventory.is_worn(id(1)));
}

#[test]
fn a_full_set_of_rings_gives_up_one() {
    let mut inventory = carrying(&[ItemKind::Ring; 11]);
    for n in 1..=10 {
        inventory.wear(id(n)).unwrap();
    }
    let swapped = swap(&mut inventory, 11).unwrap();
    assert_eq!(swapped.taken_off.len(), 1);
    assert_eq!(inventory.worn.len(), 10);
}

#[test]
fn nothing_changes_when_what_would_come_off_is_covered() {
    use ItemKind::{Boots, Socks};
    let mut inventory = carrying(&[Socks, Boots, Socks]);
    inventory.wear(id(1)).unwrap();
    inventory.wear(id(2)).unwrap();
    let before = inventory.clone();
    assert_eq!(swap(&mut inventory, 3), Err(WearRefused::Covered(id(2))));
    assert_eq!(inventory, before);
}

#[test]
fn a_curse_holds_unless_in_god_mode() {
    let mut inventory = carrying(&[ItemKind::Hat]);
    inventory.worn.push(Item {
        favour: Some(Favour::Cursed),
        ..made(ItemKind::Hat, 9)
    });
    let before = inventory.clone();
    assert_eq!(swap(&mut inventory, 1), Err(WearRefused::Stuck(id(9))));
    assert_eq!(inventory, before);
    let swapped = inventory.wear_swapping(id(1), true, StackLimits::NONE);
    assert_eq!(swapped.unwrap().taken_off, [id(9)]);
}

#[test]
fn no_room_for_what_comes_off_refuses_it() {
    let mut inventory = carrying(&[]);
    inventory.worn.push(made(ItemKind::Hat, 1));
    inventory.size = (0, 0);
    let before = inventory.clone();
    let new = made(ItemKind::Hat, 2);
    let tried = inventory.wear_new_swapping(new, false, StackLimits::NONE);
    assert_eq!(tried, Err(WearRefused::NoRoom(id(1))));
    assert_eq!(inventory, before);
}

#[test]
fn other_reasons_come_before_taking_anything_off() {
    // A chicken has no head for a hat, whatever it wears.
    let mut inventory = carrying(&[ItemKind::Hat]);
    inventory.creature = CreatureKind::Chicken;
    assert!(matches!(
        swap(&mut inventory, 1),
        Err(WearRefused::WrongBody(_))
    ));
}
