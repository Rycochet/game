//! Turns fingers into what the rest of the game reads: a tap becomes a left
//! mouse click, and the point being touched becomes `PointerPosition`; drags
//! and two-finger movements go in `TouchGesture`. See
//! `docs/context/client/touch.md`.

use bevy::input::InputSystems;
use bevy::input::touch::Touches;
use bevy::prelude::*;
use bevy::window::PrimaryWindow;

use crate::common::{PointerPosition, PointerSystems, TouchGesture};

/// A finger moving further than this (logical pixels) from where it landed is
/// dragging, so lifting it is no longer a tap.
const TAP_SLOP: f32 = 10.0;

/// Two fingers closer than this (logical pixels) give no reliable pinch or
/// twist.
const MIN_FINGER_GAP: f32 = 1.0;

pub struct TouchPlugin;

impl Plugin for TouchPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            PreUpdate,
            read_touches.in_set(PointerSystems).after(InputSystems),
        );
    }
}

/// One finger on the screen, as this frame left it.
#[derive(Clone, Copy, Debug)]
struct Finger {
    id: u64,
    at: Vec2,
    start: Vec2,
}

/// What this frame's fingers add up to.
#[derive(Debug, PartialEq)]
struct Reading {
    /// The point being touched, if one finger is touching or just tapped.
    position: Option<Vec2>,
    tap: bool,
    gesture: TouchGesture,
}

/// Follows the fingers from frame to frame. Everything it decides is for one
/// "contact": from the first finger landing until none are left.
#[derive(Default)]
struct Tracker {
    /// Where the fingers were at the end of the last frame.
    last: Vec<Finger>,
    /// A second finger has joined this contact: it is not a tap or a drag.
    several: bool,
    /// A finger has slid away from where it landed.
    moved: bool,
    /// The system took the touches back (a browser gesture, say).
    canceled: bool,
}

impl Tracker {
    /// Reads one frame: the fingers now `down`, those lifted this frame
    /// (which, for a quick tap, may have landed this frame too).
    fn read(&mut self, down: &[Finger], lifted: &[Finger], canceled: bool) -> Reading {
        self.several |= down.len() + lifted.len() > 1;
        self.canceled |= canceled;
        self.moved |= down
            .iter()
            .chain(lifted)
            .any(|finger| finger.at.distance(finger.start) > TAP_SLOP);

        let tap = match lifted {
            [finger] if down.is_empty() && !self.several && !self.moved && !self.canceled => {
                Some(finger.at)
            }
            _ => None,
        };
        let gesture = self.gesture(down);
        let position = match down {
            [finger] if !self.several => Some(finger.at),
            _ => tap,
        };

        self.last = down.to_vec();
        if down.is_empty() {
            (self.several, self.moved, self.canceled) = (false, false, false);
        }
        Reading {
            position,
            tap: tap.is_some(),
            gesture,
        }
    }

    fn gesture(&self, down: &[Finger]) -> TouchGesture {
        let before = |finger: &Finger| self.last.iter().find(|last| last.id == finger.id);
        match down {
            [finger] if !self.several && self.moved => before(finger)
                .map(|last| TouchGesture {
                    drag: finger.at - last.at,
                    drag_start: finger.start,
                    ..default()
                })
                .unwrap_or_default(),
            [a, b] => match (before(a), before(b)) {
                (Some(last_a), Some(last_b)) => two_fingers(last_a.at, last_b.at, a.at, b.at),
                _ => TouchGesture::default(),
            },
            _ => TouchGesture::default(),
        }
    }
}

/// How two fingers moved from `(a0, b0)` to `(a1, b1)`.
fn two_fingers(a0: Vec2, b0: Vec2, a1: Vec2, b1: Vec2) -> TouchGesture {
    let (span0, span1) = (b0 - a0, b1 - a1);
    if span0.length() < MIN_FINGER_GAP || span1.length() < MIN_FINGER_GAP {
        return TouchGesture::default();
    }
    // Only movement both fingers share tilts: in a pinch or a twist they
    // mostly go opposite ways.
    let (dy_a, dy_b) = (a1.y - a0.y, b1.y - b0.y);
    let tilt = if dy_a.signum() == dy_b.signum() {
        dy_a.signum() * dy_a.abs().min(dy_b.abs())
    } else {
        0.0
    };
    TouchGesture {
        pinch: span1.length() / span0.length(),
        // On a screen `y` grows downwards, so a positive angle is clockwise.
        twist: span0.angle_to(span1),
        tilt,
        centre: (a1 + b1) / 2.0,
        ..default()
    }
}

/// Fingers in a steady order, as `Touches` keeps them in no particular one.
fn fingers<'a>(touches: impl Iterator<Item = &'a bevy::input::touch::Touch>) -> Vec<Finger> {
    let mut fingers: Vec<Finger> = touches
        .map(|touch| Finger {
            id: touch.id(),
            at: touch.position(),
            start: touch.start_position(),
        })
        .collect();
    fingers.sort_by_key(|finger| finger.id);
    fingers
}

/// Runs in `PreUpdate`, so the maps and the game see a tap as a click made
/// with the mouse. The click lasts one frame.
fn read_touches(
    touches: Res<Touches>,
    window: Single<&Window, With<PrimaryWindow>>,
    mut mouse: ResMut<ButtonInput<MouseButton>>,
    mut pointer: ResMut<PointerPosition>,
    mut gesture: ResMut<TouchGesture>,
    mut tracker: Local<Tracker>,
    mut clicking: Local<bool>,
) {
    if *clicking {
        mouse.release(MouseButton::Left);
        *clicking = false;
    }
    let reading = tracker.read(
        &fingers(touches.iter()),
        &fingers(touches.iter_just_released()),
        touches.any_just_canceled(),
    );
    if reading.tap {
        mouse.press(MouseButton::Left);
        *clicking = true;
    }
    pointer.0 = reading.position.or_else(|| window.cursor_position());
    *gesture = reading.gesture;
}

#[cfg(test)]
mod tests {
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
        assert_eq!(wobble.gesture, TouchGesture::default());
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
        assert_eq!(one.gesture, TouchGesture::default());
        assert_eq!(one.position, None);
        assert!(!tracker.read(&[], &[finger(2, b, b)], false).tap);
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
            .init_resource::<TouchGesture>()
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
    fn a_tap_clicks_for_one_frame_and_leaves_no_pointer() {
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

        send(&mut app, TouchPhase::Ended, 1, spot);
        app.update();
        let mouse = app.world().resource::<ButtonInput<MouseButton>>();
        assert!(mouse.just_pressed(MouseButton::Left));
        assert_eq!(app.world().resource::<PointerPosition>().0, Some(spot));

        app.update();
        let mouse = app.world().resource::<ButtonInput<MouseButton>>();
        assert!(!mouse.pressed(MouseButton::Left));
        assert_eq!(app.world().resource::<PointerPosition>().0, None);
    }
}
