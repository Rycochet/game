use super::*;

/// Bevy only finds two parameters clashing when a system starts.
#[test]
fn no_system_asks_for_what_another_of_its_parameters_has() {
    IntoSystem::into_system(give_set_edges).initialize(&mut World::new());
    IntoSystem::into_system(slide_bands).initialize(&mut World::new());
}
