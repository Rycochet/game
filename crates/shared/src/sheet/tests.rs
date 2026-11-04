use super::*;
use crate::id::Id;
use crate::item::StackLimits;
use crate::item::{Item, ItemKind};

#[test]
fn every_kind_has_stats_and_looks() {
    for kind in CreatureKind::ALL {
        let def = kind.def();
        assert!(def.stats.strength > 0, "{}", def.name);
        assert!(!def.looks.is_empty(), "{}", def.name);
        let sheet = Sheet::new(kind);
        assert_eq!(sheet.stats, def.stats);
        assert_eq!(sheet.skills.len(), def.skills.len());
        assert!(sheet.appearance.suits(kind));
    }
}

#[test]
fn a_look_must_suit_the_kind_and_be_a_real_choice() {
    let look = |which, choice| Appearance(BTreeMap::from([(which, choice)]));
    assert!(look(Trait::HairColour, 3).suits(CreatureKind::Human));
    assert_eq!(look(Trait::HairColour, 3).says(Trait::HairColour), "Red");
    assert!(!look(Trait::HairColour, 99).suits(CreatureKind::Human));
    // Chickens have no hair.
    assert!(!look(Trait::HairColour, 0).suits(CreatureKind::Chicken));
    assert!(look(Trait::FeatherColour, 1).suits(CreatureKind::Chicken));
}

#[test]
fn appearance_changes_nothing_that_matters() {
    let plain = Sheet::new(CreatureKind::Human);
    let tall = Sheet {
        appearance: Appearance(BTreeMap::from([(Trait::Height, 4)])),
        ..plain.clone()
    };
    assert_eq!(plain.capacity(Size::Medium), tall.capacity(Size::Medium));
}

/// An inventory with this many stones of 5 kg.
fn stones(count: u128) -> Inventory {
    let mut inventory = Inventory {
        size: (40, 40),
        ..Inventory::default()
    };
    for n in 0..count {
        let stone = Item::new(Id::for_test(n), ItemKind::Stone);
        inventory.give(stone, StackLimits::NONE).unwrap();
    }
    inventory
}

#[test]
fn weight_moves_a_human_through_the_loads() {
    // Strength 10 and size 3: 30 kg.
    let sheet = Sheet::new(CreatureKind::Human);
    assert_eq!(sheet.capacity(Size::Medium), 30_000);
    assert_eq!(sheet.load(&stones(3)), Load::Light);
    assert_eq!(sheet.load(&stones(4)), Load::Burdened);
    assert_eq!(sheet.load(&stones(6)), Load::Burdened);
    assert_eq!(sheet.load(&stones(7)), Load::Overloaded);
}

#[test]
fn its_own_body_doesnt_weigh_a_creature_down() {
    let sheet = Sheet::new(CreatureKind::Human);
    let carrying = stones(0);
    assert!(carrying.weight_with_creature() > sheet.capacity(Size::Medium));
    assert_eq!(sheet.load(&carrying), Load::Light);
}

#[test]
fn weight_and_kind_decide_the_speed() {
    let sheet = Sheet::new(CreatureKind::Human);
    assert_eq!(sheet.speed(&stones(1)), NORMAL_SPEED);
    assert_eq!(sheet.speed(&stones(5)), NORMAL_SPEED * 3 / 4);
    assert_eq!(sheet.speed(&stones(7)), NORMAL_SPEED * 2 / 5);
    let horse = Inventory {
        creature: CreatureKind::Horse,
        ..Inventory::default()
    };
    let sheet = Sheet::new(CreatureKind::Horse);
    assert_eq!(sheet.speed(&horse), NORMAL_SPEED * 3 / 2);
}

#[test]
fn a_sheet_reads_back_from_a_message() {
    let sheet = Sheet::new(CreatureKind::Kobold);
    let bytes = postcard::to_allocvec(&sheet).unwrap();
    assert_eq!(postcard::from_bytes::<Sheet>(&bytes).unwrap(), sheet);
}
