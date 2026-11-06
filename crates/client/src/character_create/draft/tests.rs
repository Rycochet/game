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
    let ClientMessage::CreateCharacter { name, build, .. } = draft.message() else {
        panic!("not making a character");
    };
    assert_eq!(name, "Brin");
    assert_eq!(build, Build::CENTRED);
}
