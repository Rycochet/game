use super::*;

#[test]
fn beside_a_door_includes_diagonals_and_nothing_further() {
    let at = |x, y| MapPosition { x, y };
    assert!(beside(at(1, 1), (0, 0)));
    assert!(beside(at(0, 0), (0, 0)));
    assert!(!beside(at(2, 0), (0, 0)));
}

/// Bevy only finds two parameters clashing when a system starts.
#[test]
fn no_system_asks_for_what_another_of_its_parameters_has() {
    fn starts<M>(system: impl IntoSystem<(), (), M>) {
        IntoSystem::into_system(system).initialize(&mut World::new());
    }
    starts(change_of_mind);
    starts(follow_clicked_door);
    starts(open_on_arrival);
    starts(note_gifts);
    starts(read_server);
    starts(draw_grids);
    starts(show_gift_aim);
    starts(close_on_escape);
    starts(close_when_left);
}
