//! Turns fingers into what the rest of the game reads: a tap becomes a left
//! mouse click, and the point being touched becomes `PointerPosition`; drags
//! and two-finger movements go in `TouchGesture`. See
//! `docs/context/client/touch.md`.

use bevy::input::InputSystems;
use bevy::input::touch::Touches;
use bevy::prelude::*;
use bevy::window::PrimaryWindow;

use crate::common::{
    PointerOverride, PointerPosition, PointerSystems, TouchGesture, UiPointer, in_interface,
};

/// A finger moving further than this (logical pixels) from where it landed is
/// dragging, so lifting it is no longer a tap.
const TAP_SLOP: f32 = 10.0;

/// How long a finger stays still to be holding something, in seconds.
const HOLD_SECONDS: f32 = 0.3;
/// How long what was tapped still counts as under the pointer, for its name to show,
/// in seconds.
const TAP_SHOWS_SECONDS: f32 = 3.0;

/// Two fingers closer than this (logical pixels) give no reliable pinch or
/// twist.
const MIN_FINGER_GAP: f32 = 1.0;

pub struct TouchPlugin;

impl Plugin for TouchPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            PreUpdate,
            read_touches.in_set(PointerSystems).after(InputSystems),
        )
        .configure_sets(
            PreUpdate,
            PointerOverride.in_set(PointerSystems).after(read_touches),
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
    /// The finger that landed first in this contact.
    first: Option<u64>,
    /// When this contact began.
    landed: Option<f32>,
    /// Where the last tap was, and when.
    last_tap: Option<(Vec2, f32)>,
    /// A tap's click is being held down, until the next frame.
    clicking: bool,
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
        // Fingers are in a steady order, so the first of several landing in one
        // frame is the same one each time.
        if self.first.is_none() {
            self.first = down.first().map(|finger| finger.id);
        }
        let first_finger = down
            .iter()
            .find(|finger| Some(finger.id) == self.first)
            .map(|finger| finger.at);
        let other_finger_tapped = first_finger.is_some()
            && lifted
                .iter()
                .any(|finger| finger.at.distance(finger.start) <= TAP_SLOP);
        let gesture = TouchGesture {
            first_finger,
            other_finger_tapped,
            ..self.gesture(down)
        };
        let position = match down {
            [finger] if !self.several => Some(finger.at),
            _ => tap,
        };

        self.last = down.to_vec();
        if down.is_empty() {
            (self.several, self.moved, self.canceled) = (false, false, false);
            self.first = None;
        }
        Reading {
            position,
            tap: tap.is_some(),
            gesture,
        }
    }

    /// Where the one finger down landed, if it has stayed there since `HOLD_SECONDS`
    /// ago. Call it after `read`.
    fn still_at(&mut self, down: &[Finger], now: f32) -> Option<Vec2> {
        if down.is_empty() {
            self.landed = None;
            return None;
        }
        let landed = *self.landed.get_or_insert(now);
        match down {
            [finger] if !self.several && !self.moved && now - landed >= HOLD_SECONDS => {
                Some(finger.start)
            }
            _ => None,
        }
    }

    /// What was just tapped, for a while, unless a finger is down again.
    fn tapped_point(&mut self, tap: Option<Vec2>, any_down: bool, now: f32) -> Option<Vec2> {
        if let Some(at) = tap {
            self.last_tap = Some((at, now));
        }
        if any_down
            || self
                .last_tap
                .is_some_and(|(_, when)| now - when >= TAP_SHOWS_SECONDS)
        {
            self.last_tap = None;
        }
        self.last_tap.map(|(at, _)| at)
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
#[allow(clippy::too_many_arguments)]
fn read_touches(
    touches: Res<Touches>,
    window: Single<&Window, With<PrimaryWindow>>,
    mut mouse: ResMut<ButtonInput<MouseButton>>,
    mut pointer: ResMut<PointerPosition>,
    mut ui_pointer: ResMut<UiPointer>,
    ui_scale: Res<UiScale>,
    mut gesture: ResMut<TouchGesture>,
    mut tracker: Local<Tracker>,
    time: Res<Time<Real>>,
) {
    if tracker.clicking {
        mouse.release(MouseButton::Left);
        tracker.clicking = false;
    }
    let now = time.elapsed_secs();
    let down = fingers(touches.iter());
    let reading = tracker.read(
        &down,
        &fingers(touches.iter_just_released()),
        touches.any_just_canceled(),
    );
    if reading.tap {
        mouse.press(MouseButton::Left);
        tracker.clicking = true;
    }
    let tapped = tracker.tapped_point(
        reading.position.filter(|_| reading.tap),
        !down.is_empty(),
        now,
    );
    pointer.0 = reading
        .position
        .or(tapped)
        .or_else(|| window.cursor_position());
    ui_pointer.0 = pointer.0.map(|at| in_interface(at, &ui_scale));
    *gesture = TouchGesture {
        tap: reading.tap,
        still_at: tracker.still_at(&down, now),
        ..reading.gesture
    };
}

#[cfg(test)]
mod tests;
