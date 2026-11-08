use super::*;

#[test]
fn the_last_ticked_box_is_disabled() {
    let one = Difficulties::only(Difficulty::Hard);
    assert_eq!(
        box_state(Difficulty::Hard, one, false),
        ButtonState::DisabledSelected
    );
    assert_eq!(box_state(Difficulty::Easy, one, false), ButtonState::Normal);
    let two = one.with(Difficulty::Easy);
    assert_eq!(box_state(Difficulty::Hard, two, false), ButtonState::Normal);
}

#[test]
fn the_environment_disables_them_all() {
    let two = Difficulties::only(Difficulty::Hard).with(Difficulty::Easy);
    assert_eq!(
        box_state(Difficulty::Hard, two, true),
        ButtonState::DisabledSelected
    );
    assert_eq!(
        box_state(Difficulty::Normal, two, true),
        ButtonState::Disabled
    );
}
