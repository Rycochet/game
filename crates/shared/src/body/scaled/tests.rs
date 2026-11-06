use super::*;

#[test]
fn bipeds_stand_at_their_own_height() {
    let halfling = Layout::at_size(CreatureKind::Halfling).expect("a halfling");
    let tall = halfling.chin()[1] + halfling.head.length;
    let usual = Frame::usual_height(CreatureKind::Halfling);
    assert!((tall - usual).abs() <= 5, "{tall} {usual}");
    // Beasts are already at their own.
    assert_eq!(
        Layout::at_size(CreatureKind::Horse),
        Layout::of(CreatureKind::Horse)
    );
}

#[test]
fn whole_size_is_the_same_body() {
    for kind in CreatureKind::ALL {
        if let Some(layout) = Layout::of(kind) {
            assert_eq!(layout.scaled(WHOLE), layout, "{kind:?}");
        }
    }
}
