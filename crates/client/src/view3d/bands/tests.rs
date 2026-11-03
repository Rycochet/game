use super::*;

#[test]
fn the_bands_go_the_way_of_the_change_no_faster_than_the_cap() {
    let mut speed = 0.0;
    for _ in 0..200 {
        speed = eased(speed, 100_000.0, 0.016);
    }
    assert!(speed > 0.0);
    assert!(speed <= MOST_SPEED);
    for _ in 0..400 {
        speed = eased(speed, -100_000.0, 0.016);
    }
    assert!(speed < 0.0);
    assert!(speed >= -MOST_SPEED);
}

#[test]
fn the_bands_ease_to_a_stop_after_the_view_stops() {
    let first = eased(MOST_SPEED, 0.0, 0.016);
    assert!(first > 0.0);
    assert!(first < MOST_SPEED, "not all at once");
    let mut speed = first;
    for _ in 0..500 {
        speed = eased(speed, 0.0, 0.016);
    }
    assert_eq!(speed, 0.0);
}

/// Bevy only finds two parameters clashing when a system starts.
#[test]
fn it_asks_for_nothing_twice() {
    IntoSystem::into_system(drift_bands).initialize(&mut World::new());
}
