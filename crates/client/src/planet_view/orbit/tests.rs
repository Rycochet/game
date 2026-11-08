use super::*;

const START: f32 = 6.0;

#[test]
fn it_starts_looking_at_the_globe_from_above_the_equator() {
    let place = Orbit::default().transform(START);
    assert!((place.translation.length() - START).abs() < 1e-4);
    assert!(place.translation.y > 0.0, "north of the equator");
    assert!(
        place.translation.z > 0.0,
        "where the view always looked from"
    );
    let ahead = place.forward().as_vec3();
    assert!(ahead.dot(-place.translation.normalize()) > 0.9999);
}

#[test]
fn dragging_right_swings_the_camera_west_and_down_tilts_it_north() {
    let mut orbit = Orbit::default();
    orbit.turn(Vec2::new(100.0, 0.0));
    let place = orbit.transform(START);
    assert!(place.translation.x < 0.0, "{place:?}");
    let mut orbit = Orbit::default();
    let before = orbit.pitch;
    orbit.turn(Vec2::new(0.0, 50.0));
    assert!(orbit.pitch > before);
}

#[test]
fn it_never_tilts_over_a_pole() {
    let mut orbit = Orbit::default();
    orbit.turn(Vec2::new(0.0, 100_000.0));
    assert!((orbit.pitch - MOST_PITCH_DEGREES.to_radians()).abs() < 1e-6);
    orbit.turn(Vec2::new(0.0, -100_000.0));
    assert!((orbit.pitch + MOST_PITCH_DEGREES.to_radians()).abs() < 1e-6);
    // Still a sound view from below.
    assert!(orbit.transform(START).translation.is_finite());
}

#[test]
fn zoom_stays_between_the_start_and_close_to_the_surface() {
    let mut orbit = Orbit::default();
    orbit.zoom_by(-5.0, START);
    assert_eq!(orbit.zoom, 1.0);
    orbit.zoom_by(1_000.0, START);
    let closest = orbit.transform(START).translation.length();
    assert!((closest - CLOSEST).abs() < 1e-4, "{closest}");
    let mut orbit = Orbit::default();
    orbit.zoom_by_pinch(2.0, START);
    assert!((orbit.zoom - 0.5).abs() < 1e-6);
}

#[test]
fn reset_goes_back_to_the_start() {
    let mut orbit = Orbit::default();
    orbit.turn(Vec2::new(40.0, 30.0));
    orbit.twist(1.0);
    orbit.zoom_by(3.0, START);
    orbit.reset();
    assert_eq!(orbit, Orbit::default());
}
