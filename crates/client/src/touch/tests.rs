use std::time::Duration;

use bevy::input::touch::{TouchInput, TouchPhase, touch_screen_input_system};

use super::*;

fn finger(id: u64, start: (f32, f32), at: (f32, f32)) -> Finger {
    Finger {
        id,
        at: Vec2::from(at),
        start: Vec2::from(start),
    }
}

#[test]
fn lifting_a_still_finger_is_a_tap_where_it_was() {
    let mut tracker = Tracker::default();
    let spot = (50.0, 60.0);
    let down = tracker.read(&[finger(1, spot, spot)], &[], false);
    assert!(!down.tap);
    assert_eq!(down.position, Some(Vec2::from(spot)));
    let up = tracker.read(&[], &[finger(1, spot, (53.0, 60.0))], false);
    assert!(up.tap);
    assert_eq!(up.position, Some(Vec2::new(53.0, 60.0)));
}

#[test]
fn a_tap_within_one_frame_counts() {
    let spot = (50.0, 60.0);
    let reading = Tracker::default().read(&[], &[finger(1, spot, spot)], false);
    assert!(reading.tap);
}

#[test]
fn a_slide_is_a_drag_and_not_a_tap_even_if_it_comes_back() {
    let mut tracker = Tracker::default();
    let spot = (50.0, 60.0);
    tracker.read(&[finger(1, spot, spot)], &[], false);
    let far = tracker.read(&[finger(1, spot, (80.0, 60.0))], &[], false);
    assert_eq!(far.gesture.drag, Vec2::new(30.0, 0.0));
    assert_eq!(
        tracker.read(&[finger(1, spot, spot)], &[], false).position,
        Some(Vec2::from(spot))
    );
    assert!(!tracker.read(&[], &[finger(1, spot, spot)], false).tap);
    // The next contact starts afresh.
    assert!(tracker.read(&[], &[finger(2, spot, spot)], false).tap);
}

#[test]
fn a_small_wobble_does_not_drag() {
    let mut tracker = Tracker::default();
    let spot = (50.0, 60.0);
    tracker.read(&[finger(1, spot, spot)], &[], false);
    let wobble = tracker.read(&[finger(1, spot, (54.0, 60.0))], &[], false);
    assert_eq!(wobble.gesture.drag, Vec2::ZERO);
}

#[test]
fn a_second_finger_stops_taps_and_drags_until_all_are_lifted() {
    let mut tracker = Tracker::default();
    let (a, b) = ((10.0, 10.0), (100.0, 10.0));
    tracker.read(&[finger(1, a, a)], &[], false);
    let both = tracker.read(&[finger(1, a, a), finger(2, b, b)], &[], false);
    assert_eq!(both.position, None);
    // One lifted: the other neither drags nor taps, however it moves.
    let one = tracker.read(&[finger(2, b, (150.0, 10.0))], &[finger(1, a, a)], false);
    assert_eq!(one.gesture.drag, Vec2::ZERO);
    assert_eq!(one.position, None);
    assert!(!tracker.read(&[], &[finger(2, b, b)], false).tap);
}

#[test]
fn the_first_finger_is_followed_and_a_second_tapping_is_told() {
    let mut tracker = Tracker::default();
    let (a, b) = ((10.0, 10.0), (100.0, 10.0));
    let held = tracker.read(&[finger(1, a, a)], &[], false);
    assert_eq!(held.gesture.first_finger, Some(Vec2::from(a)));
    assert!(!held.gesture.other_finger_tapped);

    let moved = (40.0, 10.0);
    let both = tracker.read(&[finger(1, a, moved), finger(2, b, b)], &[], false);
    assert_eq!(both.gesture.first_finger, Some(Vec2::from(moved)));
    let tapped = tracker.read(&[finger(1, a, moved)], &[finger(2, b, b)], false);
    assert!(tapped.gesture.other_finger_tapped);
    assert_eq!(tapped.gesture.first_finger, Some(Vec2::from(moved)));

    // A second finger sliding away first isn't a tap.
    let slid = tracker.read(
        &[finger(1, a, moved)],
        &[finger(3, b, (200.0, 10.0))],
        false,
    );
    assert!(!slid.gesture.other_finger_tapped);

    let up = tracker.read(&[], &[finger(1, a, moved)], false);
    assert_eq!(up.gesture.first_finger, None);
    assert!(!up.gesture.other_finger_tapped);
}

#[test]
fn a_canceled_touch_is_not_a_tap() {
    let mut tracker = Tracker::default();
    let spot = (50.0, 60.0);
    tracker.read(&[finger(1, spot, spot)], &[], false);
    let canceled = tracker.read(&[], &[], true);
    assert!(!canceled.tap);
    assert!(tracker.read(&[], &[finger(2, spot, spot)], false).tap);
}

/// Two fingers landing, then moving to these places.
fn two_finger_move(a: ((f32, f32), (f32, f32)), b: ((f32, f32), (f32, f32))) -> TouchGesture {
    let mut tracker = Tracker::default();
    tracker.read(&[finger(1, a.0, a.0), finger(2, b.0, b.0)], &[], false);
    tracker
        .read(&[finger(1, a.0, a.1), finger(2, b.0, b.1)], &[], false)
        .gesture
}

#[test]
fn spreading_fingers_is_a_pinch_above_one() {
    let gesture = two_finger_move(
        ((100.0, 100.0), (50.0, 100.0)),
        ((200.0, 100.0), (250.0, 100.0)),
    );
    assert!((gesture.pinch - 2.0).abs() < 1e-5);
    assert!(gesture.twist.abs() < 1e-5);
    assert_eq!(gesture.tilt, 0.0);
    assert_eq!(gesture.centre, Vec2::new(150.0, 100.0));
}

#[test]
fn turning_fingers_clockwise_on_screen_is_a_positive_twist() {
    // The right finger moves down and the left one up.
    let gesture = two_finger_move(
        ((100.0, 100.0), (100.0, 90.0)),
        ((200.0, 100.0), (200.0, 110.0)),
    );
    assert!(gesture.twist > 0.0);
    assert_eq!(gesture.tilt, 0.0);
}

#[test]
fn moving_both_fingers_down_tilts_by_what_they_share() {
    let gesture = two_finger_move(
        ((100.0, 100.0), (100.0, 120.0)),
        ((200.0, 100.0), (200.0, 130.0)),
    );
    assert_eq!(gesture.tilt, 20.0);
}

/// The whole system, fed the messages the window sends.
fn app() -> App {
    let mut app = App::new();
    app.add_message::<TouchInput>()
        .init_resource::<Touches>()
        .init_resource::<ButtonInput<MouseButton>>()
        .init_resource::<PointerPosition>()
        .init_resource::<UiPointer>()
        .insert_resource(UiScale(2.0))
        .init_resource::<TouchGesture>()
        .init_resource::<Time<Real>>()
        .add_systems(Update, (touch_screen_input_system, read_touches).chain());
    app.world_mut().spawn((Window::default(), PrimaryWindow));
    app
}

fn send(app: &mut App, phase: TouchPhase, id: u64, position: Vec2) {
    let window = app
        .world_mut()
        .query_filtered::<Entity, With<PrimaryWindow>>()
        .single(app.world())
        .expect("a window");
    app.world_mut().write_message(TouchInput {
        phase,
        position,
        window,
        force: None,
        id,
    });
}

#[test]
fn a_tap_clicks_for_one_frame_and_its_pointer_stays_a_while() {
    let mut app = app();
    let spot = Vec2::new(40.0, 30.0);
    send(&mut app, TouchPhase::Started, 1, spot);
    app.update();
    assert!(
        !app.world()
            .resource::<ButtonInput<MouseButton>>()
            .pressed(MouseButton::Left)
    );
    assert_eq!(app.world().resource::<PointerPosition>().0, Some(spot));
    // In the interface's pixels, drawn twice its size here.
    assert_eq!(app.world().resource::<UiPointer>().0, Some(spot / 2.0));

    send(&mut app, TouchPhase::Ended, 1, spot);
    app.update();
    let mouse = app.world().resource::<ButtonInput<MouseButton>>();
    assert!(mouse.just_pressed(MouseButton::Left));
    assert_eq!(app.world().resource::<PointerPosition>().0, Some(spot));

    app.update();
    let mouse = app.world().resource::<ButtonInput<MouseButton>>();
    assert!(!mouse.pressed(MouseButton::Left));
    assert_eq!(app.world().resource::<PointerPosition>().0, Some(spot));

    let later = Duration::from_secs_f32(TAP_SHOWS_SECONDS + 1.0);
    let mut time = app.world_mut().resource_mut::<Time<Real>>();
    time.update();
    time.update_with_duration(later);
    app.update();
    assert_eq!(app.world().resource::<PointerPosition>().0, None);
}

#[test]
fn only_a_finger_staying_still_is_holding() {
    let finger = |at: (f32, f32)| Finger {
        id: 1,
        at: at.into(),
        start: (10.0, 10.0).into(),
    };
    let mut tracker = Tracker::default();
    let down = [finger((10.0, 10.0))];
    tracker.read(&down, &[], false);
    assert_eq!(tracker.still_at(&down, 1.0), None);
    assert_eq!(
        tracker.still_at(&down, 2.0 + HOLD_SECONDS),
        Some((10.0, 10.0).into())
    );

    let mut tracker = Tracker::default();
    tracker.read(&down, &[], false);
    tracker.still_at(&down, 1.0);
    let slid = [finger((40.0, 10.0))];
    tracker.read(&slid, &[], false);
    assert_eq!(tracker.still_at(&slid, 2.0), None);
}
