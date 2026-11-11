//! What is held follows the pointer or finger, turns, and is let go.

use bevy::prelude::*;
use bevy::ui::{CalculatedClip, ComputedStackIndex, UiGlobalTransform};

use super::{Dropped, Hand, HeldObject, HeldOver, HeldWhat};
use crate::common::{Action, Preferences, TouchGesture, UiPointer, in_interface};
use crate::dom::object_grid::GridBox;
use crate::widgets::{CELL, Floating, logical_rect, shown_at};

/// A press and release this quick, this close together, is a click: what it
/// picked up stays held until the next.
pub const CLICK_SECONDS: f64 = 0.3;
pub const CLICK_PIXELS: f32 = 8.0;

/// Whether a press and release were a click rather than a drag.
pub fn is_click(from: Vec2, to: Vec2, seconds: f64) -> bool {
    seconds < CLICK_SECONDS && from.distance(to) < CLICK_PIXELS
}

type Rect<'a> = (
    &'a ComputedStackIndex,
    &'a ComputedNode,
    &'a UiGlobalTransform,
    &'a InheritedVisibility,
    Option<&'a CalculatedClip>,
);

/// Turns what is held, moves it to the pointer or finger, says what it is
/// over, and lets go of it when its hand does.
#[allow(clippy::too_many_arguments)]
pub(super) fn follow_held(
    mut held: ResMut<HeldObject>,
    (pointer, ui_scale): (Res<UiPointer>, Res<UiScale>),
    mut mouse: ResMut<ButtonInput<MouseButton>>,
    mut gesture: ResMut<TouchGesture>,
    (mut keys, preferences): (ResMut<ButtonInput<KeyCode>>, Res<Preferences>),
    time: Res<Time<Real>>,
    grids: Query<(Entity, &GridBox)>,
    nodes: Query<(Entity, Rect), Without<Floating>>,
    mut dropped: MessageWriter<Dropped>,
    mut was_down: Local<bool>,
) {
    // Widgets may have taken `just_pressed`.
    let new_press = mouse.pressed(MouseButton::Left) && !*was_down;
    *was_down = mouse.pressed(MouseButton::Left);
    let Some(mut thing) = held.thing.take() else {
        held.at = None;
        held.over = HeldOver::default();
        return;
    };

    let turn = mouse.just_pressed(MouseButton::Right) || gesture.other_finger_tapped;
    // So the camera doesn't turn with the same press.
    mouse.reset(MouseButton::Right);
    let controls = &preferences.controls;
    if (turn || controls.clear_just_pressed(&mut keys, Action::TurnItem))
        && let HeldWhat::Item(item) = &mut thing.what
    {
        item.turn();
    }
    let finger = gesture.first_finger.map(|at| in_interface(at, &ui_scale));
    if thing.by == Hand::Finger {
        // Two fingers on the screen here are holding and turning, not moving
        // the camera.
        *gesture = TouchGesture::default();
    }
    let at = match thing.by {
        Hand::Finger => finger.or(held.at),
        Hand::Mouse | Hand::Clicked | Hand::Kept => pointer.0.or(held.at),
    };
    held.at = at;
    let over = at.map(|at| over(at, &grids, &nodes)).unwrap_or_default();
    held.over = over;

    let released = !mouse.pressed(MouseButton::Left);
    let now = time.elapsed_secs_f64();
    if thing.by == Hand::Mouse
        && released
        && let (Some((from, when)), Some(at)) = (held.picked, at)
        // In the window's pixels, so a click is the same whatever the size.
        && is_click(from * ui_scale.0, at * ui_scale.0, now - when)
    {
        thing.by = Hand::Clicked;
    }
    let let_go = match thing.by {
        Hand::Mouse => released,
        Hand::Clicked => new_press,
        Hand::Finger => finger.is_none(),
        Hand::Kept => false,
    };
    if let_go {
        dropped.write(Dropped { thing, at, over });
    } else {
        held.thing = Some(thing);
    }
}

/// What is in front at this point: any node, and the grid of things there.
fn over(
    at: Vec2,
    grids: &Query<(Entity, &GridBox)>,
    nodes: &Query<(Entity, Rect), Without<Floating>>,
) -> HeldOver {
    let cell = grids
        .iter()
        .filter_map(|(grid, GridBox(cells))| {
            let (_, (stack, node, transform, visible, _)) = nodes.get(*cells).ok()?;
            let inside = node.contains_point(*transform, at / node.inverse_scale_factor);
            (visible.get() && inside).then_some((grid, stack.0, node, transform))
        })
        .max_by_key(|(_, stack, ..)| *stack)
        .map(|(grid, _, node, transform)| {
            let (corner, _) = logical_rect(node, transform);
            let cell = ((at - corner) / CELL).floor();
            (grid, cell.x as i32, cell.y as i32)
        });
    // Only what is in front counts.
    let front = nodes
        .iter()
        .filter(|(_, (_, node, transform, visible, clip))| {
            visible.get() && shown_at(at, node, transform, *clip)
        })
        .max_by_key(|(_, (stack, ..))| stack.0)
        .map(|(entity, _)| entity);
    HeldOver { front, cell }
}
