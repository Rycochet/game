use super::*;

#[test]
fn prizes_have_quality_and_can_be_kept() {
    for kind in PRIZE_KINDS {
        let def = kind.def();
        assert!(def.has_quality && !def.counted, "{kind:?}");
        assert!(def.container.is_none(), "{kind:?}");
    }
}

#[test]
fn a_planet_always_has_the_same_set_item() {
    assert_eq!(set_item_of(42), set_item_of(42));
    let kinds: std::collections::HashSet<_> =
        (0..200).map(|planet| set_item_of(planet).0).collect();
    assert!(kinds.len() > 10, "planets' set items vary");
    for planet in 0..200 {
        let (_, quality) = set_item_of(planet);
        assert!(quality != Quality::Poor && quality != Quality::Artifact);
    }
}
