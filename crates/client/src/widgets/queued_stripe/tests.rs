use super::*;

/// Bevy only finds two parameters clashing when a system starts.
#[test]
fn no_system_asks_for_what_another_of_its_parameters_has() {
    IntoSystem::into_system(give_queued_stripes).initialize(&mut World::new());
    IntoSystem::into_system(chase_stripes).initialize(&mut World::new());
}

#[test]
fn the_stripes_move_on_and_stay_within_one_pair() {
    assert_eq!(chased(0.0, 0.1), SPEED * 0.1);
    let later = chased(STRIPE_PIXELS - 1.0, 1.0);
    assert!((0.0..STRIPE_PIXELS).contains(&later));
}
