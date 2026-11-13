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
            Attributes::for_kind(kind, crate::gender::Gender::Neutral, &Build::CENTRED)
        );
        assert_eq!(sheet.skills, crate::skill::Skills::starting(kind));
        assert!(sheet.appearance.suits(kind, Gender::Neutral));
    }
}

#[test]
fn a_look_must_suit_the_kind_and_be_a_real_choice() {
    let look = |which, choice| Appearance(BTreeMap::from([(which, choice)]));
    assert!(look(Trait::HairColour, 3).suits(CreatureKind::Human, Gender::Neutral));
    assert_eq!(
        look(Trait::HairColour, 3).says(CreatureKind::Human, Trait::HairColour),
        "Red"
    );
    assert!(!look(Trait::HairColour, 99).suits(CreatureKind::Human, Gender::Neutral));
    // Chickens have no hair.
    assert!(!look(Trait::HairColour, 0).suits(CreatureKind::Chicken, Gender::Neutral));
    assert!(look(Trait::FeatherColour, 1).suits(CreatureKind::Chicken, Gender::Neutral));
}

#[test]
fn a_female_has_no_beard() {
    let bearded = Appearance(BTreeMap::from([(Trait::Beard, 2), (Trait::HairColour, 1)]));
    let human = CreatureKind::Human;
    assert!(bearded.suits(human, Gender::Male));
    assert!(bearded.suits(human, Gender::Neutral));
    assert!(!bearded.suits(human, Gender::Female));
    assert!(!Trait::Beard.applies(human, Gender::Female));
    assert!(Trait::Beard.applies(CreatureKind::Dwarf, Gender::Male));
    // Fitting drops only the beard, and only for a female.
    let fitted = bearded.clone().fitted(Gender::Female);
    assert!(fitted.suits(human, Gender::Female));
    assert_eq!(fitted.0.get(&Trait::HairColour), Some(&1));
    assert_eq!(fitted.hair(human).beard, crate::body::Beard::None);
    assert_eq!(bearded.clone().fitted(Gender::Male), bearded);
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
        inventory
            .give(stone, StackLimits::NONE, &mut crate::id::stand_ins())
            .unwrap();
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
            if !matches!(
                which,
                Trait::Height | Trait::HairStyle | Trait::Beard | Trait::Markings
            ) {
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

#[test]
fn a_save_from_before_genders_is_neutral_and_keeps_its_attributes() {
    // Attributes with no gender's bonus in them, as saved before genders.
    let grown = median_human();
    let saved = ron::to_string(&grown).unwrap();
    let old = saved.replace("gender:Neutral,", "");
    assert_ne!(old, saved, "the field should be in the save: {saved}");
    let loaded: Sheet = ron::from_str(&old).unwrap();
    assert_eq!(loaded.gender, crate::gender::Gender::Neutral);
    let settled = loaded.settled(CreatureKind::Human);
    assert_eq!(settled.attributes, Attributes::all(100));
}

#[test]
fn a_save_from_before_alignments_starts_at_zero() {
    let grown = median_human();
    let saved = ron::to_string(&grown).unwrap();
    let old = saved.replace("alignment:Some((0)),", "");
    assert_ne!(old, saved, "the field should be in the save: {saved}");
    let loaded: Sheet = ron::from_str(&old).unwrap();
    assert_eq!(loaded.alignment, None);
    let settled = loaded.settled(CreatureKind::Human);
    assert_eq!(
        settled.alignment,
        Some(crate::alignment::Alignment::NEUTRAL)
    );
}

#[test]
fn a_sheet_as_sent_keeps_its_alignment_secret() {
    let mut sheet = median_human();
    sheet.alignment = Some(crate::alignment::Alignment::new(-450));
    let sent = sheet.as_sent();
    assert_eq!(sent.alignment, None);
    assert_eq!(
        Sheet {
            alignment: None,
            ..sheet
        },
        sent
    );
}

#[test]
fn markings_are_a_look_older_saves_have_the_first_of() {
    // A look saved before markings has none chosen: a wolf then has its first, a
    // pale belly and mask; cattle the first, plain.
    let old = Appearance::default();
    let wolf = old.marks(CreatureKind::Wolf).unwrap();
    assert!(wolf.belly && wolf.face == Face::Mask);
    assert_eq!(old.marks(CreatureKind::Cattle), None);
    // Only kinds with markings to choose have the look, and some cattle are patched
    // in different layouts.
    for kind in CreatureKind::ALL {
        let has = kind.def().looks.contains(&Trait::Markings);
        assert_eq!(
            has,
            !Trait::Markings.choices_for(kind).is_empty(),
            "{kind:?}"
        );
    }
    let layouts: std::collections::BTreeSet<_> = Trait::Markings
        .choices_for(CreatureKind::Cattle)
        .iter()
        .filter_map(|choice| match choice.pattern {
            Pattern::Marked(marks) => marks.patches,
            _ => None,
        })
        .collect();
    assert!(layouts.len() > 4);
}
