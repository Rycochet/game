use super::*;
use crate::blade::Hands;
use crate::creature::CreatureKind;
use crate::id::Id;
use crate::item::{Inventory, Item, ItemKind, Quality, StackLimits};
use crate::item_form::{ItemForm, ItemOrder, make_item};
use crate::rng::Seed;
use crate::wear::WearRefused;

/// A blade of this kind, rolled for a human, with id `n` and a hilt for `hands`.
fn blade(kind: ItemKind, n: u128, hands: Hands) -> Item {
    let order = ItemOrder::plain(kind, Quality::Common);
    let mut new_id = || Id::for_test(1000 + n);
    let mut item = make_item(
        &order,
        CreatureKind::Human,
        Id::for_test(n),
        Seed::new(n as u64),
        None,
        &mut new_id,
    );
    if let Some(ItemForm::Blade(form)) = &mut item.form {
        form.hands = hands;
    }
    item
}

fn carrying(kind: CreatureKind, items: Vec<Item>) -> Inventory {
    let mut inventory = Inventory {
        creature: kind,
        size: (40, 40),
        ..Inventory::default()
    };
    for item in items {
        inventory.give(item, StackLimits::NONE).expect("room");
    }
    inventory
}

#[test]
fn nothing_held_is_bare_hands() {
    let inventory = carrying(CreatureKind::Human, vec![]);
    let wielded = Wielded::of(&inventory);
    assert_eq!(wielded.main, Weapon::UNARMED);
    assert!(!wielded.is_dual());
}

#[test]
fn the_bigger_weapon_is_the_main_hand_whichever_went_on_first() {
    let dagger = blade(ItemKind::Dagger, 1, Hands::One);
    let sword = blade(ItemKind::Sword, 2, Hands::One);
    let mut inventory = carrying(CreatureKind::Human, vec![dagger, sword]);
    inventory.wear(Id::for_test(1)).unwrap();
    inventory.wear(Id::for_test(2)).unwrap();
    let wielded = Wielded::of(&inventory);
    assert!(wielded.is_dual());
    assert_eq!(wielded.main.family, Some(crate::blade::Family::Sword));
    assert_eq!(
        wielded.off.unwrap().family,
        Some(crate::blade::Family::Dagger)
    );
    // Hands take turns, the main first.
    assert!(!wielded.for_attack(0).1);
    assert!(wielded.for_attack(1).1);
    assert!(!wielded.for_attack(2).1);
}

#[test]
fn two_hands_hold_two_one_handed_weapons_and_no_more() {
    let items = vec![
        blade(ItemKind::Dagger, 1, Hands::One),
        blade(ItemKind::Dagger, 2, Hands::One),
        blade(ItemKind::Knife, 3, Hands::One),
    ];
    let mut inventory = carrying(CreatureKind::Human, items);
    inventory.wear(Id::for_test(1)).unwrap();
    inventory.wear(Id::for_test(2)).unwrap();
    assert!(matches!(
        inventory.wear(Id::for_test(3)),
        Err(WearRefused::Clash(_))
    ));
}

#[test]
fn a_two_handed_weapon_takes_both_hands() {
    let items = vec![
        blade(ItemKind::Polearm, 1, Hands::Two),
        blade(ItemKind::Dagger, 2, Hands::One),
    ];
    let mut inventory = carrying(CreatureKind::Human, items);
    inventory.wear(Id::for_test(1)).unwrap();
    assert!(matches!(
        inventory.wear(Id::for_test(2)),
        Err(WearRefused::Clash(_))
    ));
    let wielded = Wielded::of(&inventory);
    assert!(!wielded.is_dual());
    // And not into a hand already holding something.
    let items = vec![
        blade(ItemKind::Dagger, 2, Hands::One),
        blade(ItemKind::Polearm, 1, Hands::Two),
    ];
    let mut inventory = carrying(CreatureKind::Human, items);
    inventory.wear(Id::for_test(2)).unwrap();
    assert!(inventory.wear(Id::for_test(1)).is_err());
}

#[test]
fn kinds_without_hands_fight_unarmed() {
    let dagger = blade(ItemKind::Dagger, 1, Hands::One);
    for kind in [
        CreatureKind::Horse,
        CreatureKind::Wolf,
        CreatureKind::Chicken,
    ] {
        let mut inventory = carrying(kind, vec![dagger.clone()]);
        assert!(inventory.wear(Id::for_test(1)).is_err(), "{kind:?}");
        assert_eq!(Wielded::of(&inventory).main, Weapon::UNARMED);
    }
}
