//! Targeting a creature by pointing at it: a right click, or a long press of a
//! finger.

use bevy::prelude::*;

use crate::common::{Other, PointerPosition, Target, TouchGesture};
use crate::view3d::HoveredToken;

/// How far the pointer may move between pressing and letting go of the right
/// button for it to count as a click rather than turning the camera, in pixels.
/// (A right press is to open a menu of choices later, targeting among them.)
const CLICK_SLOP: f32 = 5.0;

/// How long a finger stays still on a creature to target it, in seconds.
const LONG_PRESS: f32 = 0.5;

/// A right click on a creature targets it. Pressing and dragging turns the
/// camera instead.
pub(super) fn pick_by_right_click(
    mouse: Res<ButtonInput<MouseButton>>,
    pointer: Res<PointerPosition>,
    hovered: Res<HoveredToken>,
    others: Query<&Other>,
    mut pressed_at: Local<Option<Vec2>>,
    mut target: ResMut<Target>,
) {
    if mouse.just_pressed(MouseButton::Right) {
        *pressed_at = pointer.0;
    }
    if !mouse.just_released(MouseButton::Right) {
        return;
    }
    let still = match (pressed_at.take(), pointer.0) {
        (Some(from), Some(to)) => from.distance(to) <= CLICK_SLOP,
        _ => false,
    };
    if let Some(other) = hovered
        .0
        .filter(|_| still)
        .and_then(|token| others.get(token).ok())
    {
        target.set_if_neq(Target(Some(other.id)));
    }
}

/// How a finger held on the map is going.
#[derive(Default)]
pub(super) struct Press {
    /// How long it has been still.
    held: f32,
    /// It targeted someone, so lifting it isn't a tap on the map.
    targeted: bool,
}

/// A finger held still on a creature for `LONG_PRESS` targets it, and the tap
/// its lifting makes is taken away (`eat_tap`), so it doesn't walk there.
pub(super) fn pick_by_long_press(
    time: Res<Time>,
    gesture: Res<TouchGesture>,
    hovered: Res<HoveredToken>,
    others: Query<&Other>,
    mut press: ResMut<LongPress>,
    mut target: ResMut<Target>,
) {
    let press = &mut press.0;
    if gesture.still_at.is_none() {
        press.held = 0.0;
        return;
    }
    let before = press.held;
    press.held += time.delta_secs();
    if before >= LONG_PRESS || press.held < LONG_PRESS {
        return;
    }
    if let Some(other) = hovered.0.and_then(|token| others.get(token).ok()) {
        target.set_if_neq(Target(Some(other.id)));
        press.targeted = true;
    }
}

/// The finger being held, kept between `pick_by_long_press` and `eat_tap`.
#[derive(Resource, Default)]
pub(super) struct LongPress(pub(super) Press);

/// Lifting a finger that targeted someone isn't a tap on the map. Runs before
/// anything reads clicks.
pub(super) fn eat_tap(
    gesture: Res<TouchGesture>,
    mut mouse: ResMut<ButtonInput<MouseButton>>,
    mut press: ResMut<LongPress>,
) {
    let press = &mut press.0;
    if gesture.tap && press.targeted {
        mouse.clear_just_pressed(MouseButton::Left);
        press.targeted = false;
    } else if gesture.first_finger.is_none() {
        press.targeted = false;
    }
}
