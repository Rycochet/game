use super::*;

#[test]
fn a_clockwise_twist_turns_the_same_way_as_dragging_left() {
    // Dragging right swings the near side right: anticlockwise on screen.
    let (mut twisted, mut dragged) = (camera(40.0), camera(40.0));
    twisted.twist(0.1);
    dragged.turn(Vec2::new(-10.0, 0.0));
    assert!(twisted.yaw > camera(40.0).yaw);
    assert!(dragged.yaw > camera(40.0).yaw);
}

#[test]
fn a_pinch_keeps_to_the_zoom_limits() {
    let mut camera = camera(0.0);
    camera.zoom_by_pinch(0.5);
    assert_eq!(camera.zoom(), 1.0);
    camera.zoom_by_pinch(1.25);
    assert!((camera.zoom() - 0.8).abs() < 1e-5);
    camera.zoom_by_pinch(100.0);
    assert!((camera.zoom() - MIN_ZOOM).abs() < 1e-5);
}

fn camera(yaw_degrees: f32) -> OrbitCamera {
    let mut camera = OrbitCamera::new(30f32.to_radians(), 100.0, 0.0);
    camera.yaw = yaw_degrees.to_radians();
    camera
}

#[test]
fn default_view_is_from_the_south() {
    let transform = camera(0.0).transform(Vec3::ZERO);
    assert!(transform.translation.z > 0.0);
    assert!(transform.translation.x.abs() < 1e-3);
    assert!((transform.translation.length() - 100.0).abs() < 1e-3);
}

#[test]
fn quarter_turns_match_the_side_the_camera_is_on() {
    // Camera east of the target, looking west: west is at the top.
    assert!(camera(90.0).transform(Vec3::ZERO).translation.x > 0.0);
    assert_eq!(camera(90.0).quarter_turns(), 3);
    assert_eq!(camera(0.0).quarter_turns(), 0);
    assert_eq!(camera(-90.0).quarter_turns(), 1);
    assert_eq!(camera(180.0).quarter_turns(), 2);
    assert_eq!(camera(350.0).quarter_turns(), 0);
}

#[test]
fn ground_axes_point_right_and_towards_the_camera() {
    let camera = camera(90.0);
    let (right, towards) = camera.ground_axes();
    let position = camera.transform(Vec3::ZERO).translation;
    assert!(towards.dot(Vec2::new(position.x, position.z)) > 0.0);
    // Looking west, right is north (−z).
    assert!((right - Vec2::new(0.0, -1.0)).length() < 1e-5);
}

#[test]
fn tilt_and_zoom_stay_in_range_and_reset() {
    let mut camera = camera(0.0);
    camera.turn(Vec2::new(0.0, 10_000.0));
    assert_eq!(camera.pitch, MAX_PITCH_DEGREES.to_radians());
    camera.zoom_by(-100.0);
    assert_eq!(camera.distance, 100.0);
    camera.zoom_by(100.0);
    assert!((camera.zoom() - MIN_ZOOM).abs() < 1e-6);
    camera.reset();
    assert_eq!((camera.pitch, camera.distance), (30f32.to_radians(), 100.0));
}

#[test]
fn tilting_while_raised_starts_from_what_is_shown() {
    let mut camera = OrbitCamera::new(20f32.to_radians(), 400.0, 0.0);
    camera.set_raise(10f32.to_radians());
    assert!((camera.shown_pitch() - 30f32.to_radians()).abs() < 1e-5);
    // Turning alone keeps the setting.
    camera.turn(Vec2::new(10.0, 0.0));
    assert!((camera.pitch - 20f32.to_radians()).abs() < 1e-5);
    // Tilting takes what is shown as the setting, then tilts.
    camera.turn(Vec2::new(0.0, -10.0));
    let expected = 30f32.to_radians() - 10.0 * RADIANS_PER_PIXEL;
    assert!((camera.pitch - expected).abs() < 1e-5);
    assert_eq!(camera.raise(), 0.0);
    assert!((camera.shown_pitch() - expected).abs() < 1e-5);
}
