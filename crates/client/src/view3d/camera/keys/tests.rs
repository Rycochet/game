use super::*;

fn turn(keys: &ButtonInput<KeyCode>, seconds: f32) -> f32 {
    amount(
        &Controls::default(),
        keys,
        (Action::TurnLeft, Action::TurnRight),
        TURN,
        seconds,
    )
}

#[test]
fn a_press_steps_then_holding_keeps_going_and_letting_go_stops() {
    let mut keys = ButtonInput::<KeyCode>::default();
    keys.press(KeyCode::Numpad9);
    assert_eq!(turn(&keys, 0.5), TURN_STEP_DEGREES);
    // The next frame: still held, no longer just pressed.
    keys.clear();
    assert_eq!(turn(&keys, 0.5), TURN_DEGREES_PER_SECOND * 0.5);
    keys.release(KeyCode::Numpad9);
    keys.clear();
    assert_eq!(turn(&keys, 0.5), 0.0);
}

#[test]
fn left_turns_the_other_way_and_both_cancel_out() {
    let mut keys = ButtonInput::<KeyCode>::default();
    keys.press(KeyCode::Numpad7);
    assert_eq!(turn(&keys, 0.1), -TURN_STEP_DEGREES);
    keys.press(KeyCode::Numpad9);
    assert_eq!(turn(&keys, 0.1), 0.0);
}

#[test]
fn zoom_in_moves_the_camera_closer() {
    let mut keys = ButtonInput::<KeyCode>::default();
    keys.press(KeyCode::NumpadAdd);
    let notches = amount(
        &Controls::default(),
        &keys,
        (Action::ZoomOut, Action::ZoomIn),
        ZOOM,
        0.1,
    );
    let mut camera = OrbitCamera::new(30f32.to_radians(), 100.0, 0.0);
    camera.zoom_by(notches);
    assert!(camera.zoom() < 1.0);
}
