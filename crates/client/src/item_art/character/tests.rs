use super::*;

/// Every kind standing on two legs is drawn as itself even with nothing on; the
/// others (centaurs and animals) never are, and keep their icons.
#[test]
fn only_kinds_with_a_frame_have_a_look_bare() {
    for kind in CreatureKind::ALL {
        let look = character_look(kind, &Appearance::default(), &[]);
        assert_eq!(look.is_some(), Frame::of(kind).is_some(), "{kind:?}");
    }
}

#[test]
fn centaurs_and_animals_keep_their_icons() {
    for kind in [
        CreatureKind::Centaur,
        CreatureKind::Chicken,
        CreatureKind::Horse,
    ] {
        assert!(
            character_look(kind, &Appearance::default(), &[]).is_none(),
            "{kind:?}"
        );
    }
}
