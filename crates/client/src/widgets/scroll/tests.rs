use super::*;

#[test]
fn a_bars_ends_are_an_areas_ends() {
    assert_eq!(at_share(-0.5, 200.0), 0.0);
    assert_eq!(at_share(0.5, 200.0), 100.0);
    assert_eq!(at_share(1.5, 200.0), 200.0);
}

/// Bevy only finds two parameters clashing when a system starts.
#[test]
fn no_system_asks_for_what_another_of_its_parameters_has() {
    IntoSystem::into_system(scroll_areas).initialize(&mut World::new());
    IntoSystem::into_system(show_scroll_bars).initialize(&mut World::new());
}
