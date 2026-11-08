use super::*;
use crate::id::Id;
use crate::item::StackLimits;
use crate::item::{Item, ItemKind};

#[test]
fn every_kind_has_attributes_and_looks() {
    for kind in CreatureKind::ALL {
        let def = kind.def();
        for which in Attribute::ALL {
            assert!(def.attributes.get(which) > 0, "{} {which:?}", def.name);
        }
        assert!(!def.looks.is_empty(), "{}", def.name);
        let sheet = Sheet::new(kind);
        assert_eq!(sheet.build, Build::CENTRED);
        assert_eq!(
            sheet.attributes,
            Attributes::for_kind(kind, &Build::CENTRED)
        );
        assert_eq!(sheet.skills, crate::skill::Skills::starting(kind));
        assert!(sheet.appearance.suits(kind));
    }
}

#[test]
fn a_look_must_suit_the_kind_and_be_a_real_choice() {
    let look = |which, choice| Appearance(BTreeMap::from([(which, choice)]));
    assert!(look(Trait::HairColour, 3).suits(CreatureKind::Human));
    assert_eq!(
        look(Trait::HairColour, 3).says(CreatureKind::Human, Trait::HairColour),
        "Red"
    );
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

/// A human with every attribute at the median.
fn median_human() -> Sheet {
    Sheet {
        attributes: Attributes::all(100),
        ..Sheet::new(CreatureKind::Human)
    }
}

#[test]
fn a_save_from_before_the_nine_works_them_out() {
    // An old save has none, and its build defaults to the centre.
    let old = Sheet {
        attributes: Attributes::default(),
        build: Build::default(),
        ..Sheet::new(CreatureKind::Dwarf)
    };
    assert_eq!(
        old.settled(CreatureKind::Dwarf),
        Sheet::new(CreatureKind::Dwarf)
    );
    // One that has them keeps them.
    let grown = median_human();
    assert_eq!(grown.clone().settled(CreatureKind::Human), grown);
}

#[test]
fn weight_moves_a_human_through_the_loads() {
    // Median strength (100) and size 3: 30 kg.
    let sheet = median_human();
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
    let sheet = median_human();
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

#[test]
fn each_kind_draws_in_its_own_colours() {
    let plain = Appearance::default();
    // The first skin is the plain shade bare bodies were always drawn in.
    assert_eq!(plain.skin(CreatureKind::Human), Some([199, 184, 163]));
    assert_eq!(plain.skin(CreatureKind::Goblin), Some([112, 150, 72]));
    // A minotaur's body is its coat; a birdfolk has no skin choice, but feathers.
    assert_eq!(plain.skin(CreatureKind::Minotaur), Some([104, 70, 46]));
    assert_eq!(plain.skin(CreatureKind::Birdfolk), None);
    assert_eq!(
        plain.feathers(CreatureKind::Birdfolk),
        Some([242, 242, 238])
    );
    // Every trait has choices, and every colour trait's choices are all colours (all
    // but height and hair's shapes),
    // a hen's speckles too (a colour with specks over it).
    for kind in CreatureKind::ALL {
        for &which in kind.def().looks {
            let choices = which.choices_for(kind);
            assert!(!choices.is_empty(), "{kind:?} {which:?}");
            if !matches!(which, Trait::Height | Trait::HairStyle | Trait::Beard) {
                assert!(
                    choices.iter().all(|c| c.colour.is_some()),
                    "{kind:?} {which:?}"
                );
            }
        }
    }
    let speckled = Trait::FeatherColour.choices_for(CreatureKind::Chicken)[2];
    assert!(matches!(speckled.pattern, Pattern::Speckled(_)));
}

#[test]
fn weight_training_adds_its_value_to_what_can_be_carried() {
    let mut sheet = median_human();
    let node = crate::movement::MOVEMENT_SKILLS.weight_training;
    // 50%: a half again.
    sheet.skills.practise(node, 125_000);
    assert_eq!(sheet.capacity(Size::Medium), 45_000);
    // 100% and past it: twice as much, and no more.
    sheet.skills.practise(node, 2_000_000);
    assert_eq!(sheet.capacity(Size::Medium), 60_000);
    assert_eq!(sheet.load(&stones(7)), Load::Burdened);
}
