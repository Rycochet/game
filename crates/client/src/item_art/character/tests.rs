use super::*;

/// Every kind shows its body, even with nothing on: animals bare, people and taurs as
/// characters.
#[test]
fn every_kind_has_a_look_bare() {
    for kind in CreatureKind::ALL {
        let look = character_look(kind, &Appearance::default(), &[]);
        assert_eq!(look.is_some(), Layout::of(kind).is_some(), "{kind:?}");
    }
}

#[test]
fn animals_are_their_bare_bodies_and_centaurs_characters() {
    for kind in [CreatureKind::Chicken, CreatureKind::Horse] {
        let look = character_look(kind, &Appearance::default(), &[]);
        assert!(matches!(look, Some(Look::Body { .. })), "{kind:?}");
    }
    let centaur = character_look(CreatureKind::Centaur, &Appearance::default(), &[]);
    assert!(matches!(centaur, Some(Look::Character { .. })));
}
