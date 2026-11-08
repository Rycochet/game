use super::*;

const CELL: f32 = 24.0;

/// Looking north from the south, 20° up, from 400 units away.
fn camera() -> OrbitCamera {
    OrbitCamera::new(20f32.to_radians(), 400.0, 0.0)
}

fn shown(orbit: &OrbitCamera, raise: f32) -> Vec3 {
    orbit
        .transform_at(orbit.pitch + raise, Vec3::ZERO)
        .translation
}

#[test]
fn flat_ground_needs_no_raise() {
    let raise = clearing_raise(&camera(), Vec3::ZERO, CELL, |_, _| Some(0.0));
    assert_eq!(raise, 0.0);
}

#[test]
fn unknown_ground_is_passed_over() {
    let raise = clearing_raise(&camera(), Vec3::ZERO, CELL, |_, _| None);
    assert_eq!(raise, 0.0);
}

#[test]
fn a_slope_rising_behind_raises_the_camera_over_it() {
    let orbit = camera();
    // Rising half a cell a cell towards the camera (south), from 2 cells out.
    let ground = |_: i64, y: i64| Some((y - 2).max(0) as f32 * CELL * 0.5);
    let raise = clearing_raise(&orbit, Vec3::ZERO, CELL, ground);
    assert!(raise > 0.0);
    let head = Vec3::Y * HEAD * CELL;
    assert!(is_clear(head, shown(&orbit, raise), CELL, &ground));
    assert!(!is_clear(head, shown(&orbit, 0.0), CELL, &ground));
    // Only as much as needed: a step less is blocked.
    let less = raise - RAISE_STEP_DEGREES.to_radians();
    assert!(!is_clear(head, shown(&orbit, less), CELL, &ground));
}

#[test]
fn a_wall_of_land_in_between_is_looked_over() {
    let orbit = camera();
    // A row of land 4 cells high, 3 cells south of the player.
    let ground = |_: i64, y: i64| Some(if y == 3 { 4.0 * CELL } else { 0.0 });
    let raise = clearing_raise(&orbit, Vec3::ZERO, CELL, ground);
    assert!(raise > 0.0);
    assert!(is_clear(
        Vec3::Y * HEAD * CELL,
        shown(&orbit, raise),
        CELL,
        &ground
    ));
}

#[test]
fn land_too_high_stops_at_the_steepest_tilt() {
    let orbit = camera();
    let ground = |_: i64, y: i64| Some(if y >= 1 { 1000.0 } else { 0.0 });
    let raise = clearing_raise(&orbit, Vec3::ZERO, CELL, ground);
    assert!((orbit.pitch + raise - MAX_PITCH_DEGREES.to_radians()).abs() < 1e-5);
}

#[test]
fn easing_goes_part_way_unless_snapped() {
    assert_eq!(eased(0.0, 1.0, 0.1, true), 1.0);
    let part = eased(0.0, 1.0, 0.1, false);
    assert!(part > 0.3 && part < 0.7, "{part}");
    assert!(eased(1.0, 0.0, 0.1, false) < 1.0, "eases back down too");
}
