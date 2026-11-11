//! Dragging a locked popup by its top, with the mouse or a finger. Nothing
//! shows where to take hold of it (owner: the pointer will say, later).

use bevy::prelude::*;
use bevy::ui::UiGlobalTransform;

use super::{Popup, PopupLock};
use crate::common::{TouchGesture, UiPointer, in_interface};
use crate::dom::attrs::Length;
use crate::dom::parts::{ControlOf, Element};
use crate::widgets::logical_rect;

/// How far down from its top a popup can be taken hold of: about a line of
/// its writing.
pub(super) const GRIP: f32 = 24.0;

/// The popup being dragged, how far into it the pointer took hold, and
/// whether a finger has it.
#[derive(Default)]
pub(super) struct Dragged(Option<(Entity, Vec2, bool)>);

/// Whether a point is on the strip a popup is dragged by.
pub(super) fn on_grip(at: Vec2, corner: Vec2, size: Vec2) -> bool {
    let strip = Vec2::new(size.x, GRIP.min(size.y));
    Rect::from_corners(corner, corner + strip).contains(at)
}

type PopupBox<'a> = (
    Entity,
    &'a Popup,
    &'a Node,
    &'a ComputedNode,
    &'a UiGlobalTransform,
);

/// A press on a locked popup's top takes hold of it, and it follows the
/// pointer or finger until let go: its element's `left` and `top` change.
#[allow(clippy::too_many_arguments)]
pub(super) fn drag_popups(
    popups: Query<PopupBox>,
    locks: Query<(&ComputedNode, &UiGlobalTransform), With<PopupLock>>,
    owners: Query<&ControlOf>,
    mut elements: Query<&mut Element>,
    mut mouse: ResMut<ButtonInput<MouseButton>>,
    (pointer, ui_scale): (Res<UiPointer>, Res<UiScale>),
    mut gesture: ResMut<TouchGesture>,
    mut dragged: Local<Dragged>,
) {
    let finger = gesture.first_finger.map(|at| in_interface(at, &ui_scale));
    // The locked popup whose top this point is on, and how far into it.
    let taken_at = |at: Vec2| {
        let on_lock = locks.iter().any(|(node, transform)| {
            let (corner, size) = logical_rect(node, transform);
            Rect::from_corners(corner, corner + size).contains(at)
        });
        let found = popups.iter().find(|(_, popup, node, computed, transform)| {
            let (corner, size) = logical_rect(computed, transform);
            popup.locked && node.display != Display::None && on_grip(at, corner, size)
        });
        let (entity, .., computed, transform) = found.filter(|_| !on_lock)?;
        Some((entity, at - logical_rect(computed, transform).0))
    };
    if dragged.0.is_none() {
        if mouse.just_pressed(MouseButton::Left)
            && let Some((popup, held)) = pointer.0.and_then(taken_at)
        {
            // Taken: nothing under it sees the press.
            mouse.clear_just_pressed(MouseButton::Left);
            dragged.0 = Some((popup, held, false));
        } else if gesture.drag != Vec2::ZERO {
            let start = in_interface(gesture.drag_start, &ui_scale);
            dragged.0 = taken_at(start).map(|(popup, held)| (popup, held, true));
        }
    }
    let Some((popup, held, by_finger)) = dragged.0 else {
        return;
    };
    let at = if by_finger {
        // Moving the popup, not scrolling or moving what is under it.
        gesture.drag = Vec2::ZERO;
        finger
    } else {
        pointer.0.filter(|_| mouse.pressed(MouseButton::Left))
    };
    let still_held = match by_finger {
        true => finger.is_some(),
        false => mouse.pressed(MouseButton::Left),
    };
    if !still_held {
        dragged.0 = None;
        return;
    }
    let (Some(at), Ok(ControlOf(element))) = (at, owners.get(popup)) else {
        return;
    };
    let Ok(mut element) = elements.get_mut(*element) else {
        dragged.0 = None;
        return;
    };
    let corner = (at - held).round();
    let (left, top) = (Length::Pixels(corner.x), Length::Pixels(corner.y));
    if (element.attrs.left, element.attrs.top) != (Some(left), Some(top)) {
        (element.attrs.left, element.attrs.top) = (Some(left), Some(top));
    }
}
