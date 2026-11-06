use super::*;

/// Every kind whose body is drawn shows it even with nothing on; the others (centaurs
/// for now) keep their icons.
#[test]
fn only_kinds_with_a_layout_have_a_look_bare() {
    for kind in CreatureKind::ALL {
        let look = character_look(kind, &Appearance::default(), &[]);
        assert_eq!(look.is_some(), Layout::of(kind).is_some(), "{kind:?}");
    }
}

#[test]
fn animals_are_their_bare_bodies_and_centaurs_keep_their_icons() {
    for kind in [CreatureKind::Chicken, CreatureKind::Horse] {
        let look = character_look(kind, &Appearance::default(), &[]);
        assert!(matches!(look, Some(Look::Body { .. })), "{kind:?}");
    }
    assert!(character_look(CreatureKind::Centaur, &Appearance::default(), &[]).is_none());
}
