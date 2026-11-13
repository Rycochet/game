use super::*;
use shared::sheet::Trait;

#[test]
fn only_playable_kinds_are_offered() {
    let kinds = Draft::playable();
    assert!(kinds.contains(&CreatureKind::Human));
    assert!(kinds.contains(&CreatureKind::Dwarf));
    assert!(!kinds.contains(&CreatureKind::Horse));
    assert!(!kinds.contains(&CreatureKind::Giant));
}

#[test]
fn a_new_kind_starts_its_looks_again() {
    let mut draft = Draft::default();
    draft.appearance.0.insert(Trait::HairColour, 3);
    draft.choose_kind(CreatureKind::Human);
    assert!(!draft.appearance.0.is_empty());
    draft.choose_kind(CreatureKind::Goblin);
    assert!(draft.appearance.0.is_empty());
}

#[test]
fn it_needs_a_name_and_sends_it_trimmed() {
    let mut draft = Draft::default();
    assert!(!draft.ready());
    draft.name = "  Brin ".into();
    assert!(draft.ready());
    let ClientMessage::CreateCharacter(made) = draft.message() else {
        panic!("not making a character");
    };
    assert_eq!(made.name, "Brin");
    assert_eq!(made.build, Build::CENTRED);
}

#[test]
fn the_gender_chosen_is_sent_and_its_bonus_shown() {
    let mut draft = Draft::default();
    assert_eq!(draft.gender, Gender::Neutral);
    draft.name = "Brin".into();
    draft.gender = Gender::Female;
    let ClientMessage::CreateCharacter(made) = draft.message() else {
        panic!("not making a character");
    };
    assert_eq!(made.gender, Gender::Female);
    // The bonus layer is the kind's and the gender's together.
    let kind = draft.kind.def().bonus();
    let bonus = draft.bonus();
    assert_eq!(
        bonus.intellect,
        kind.intellect + shared::gender::GENDER_BONUS
    );
    assert_eq!(bonus.strength, kind.strength);
}

#[test]
fn choosing_female_drops_a_chosen_beard_and_nothing_else() {
    let mut draft = Draft::default();
    draft.choose_gender(Gender::Male);
    draft.appearance.0.insert(Trait::Beard, 2);
    draft.appearance.0.insert(Trait::HairColour, 3);
    draft.choose_gender(Gender::Neutral);
    assert_eq!(draft.appearance.0.get(&Trait::Beard), Some(&2));
    draft.choose_gender(Gender::Female);
    assert_eq!(draft.appearance.0.get(&Trait::Beard), None);
    assert_eq!(draft.appearance.0.get(&Trait::HairColour), Some(&3));
    assert!(draft.appearance.suits(draft.kind, draft.gender));
}
