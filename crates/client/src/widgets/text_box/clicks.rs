//! Placing the caret and selecting with the pointer: a click puts the caret,
//! a drag selects, Shift+click stretches the selection, a double click takes a
//! word and a triple click everything. Taps arrive as clicks, so fingers get
//! the same; a finger dragging over the box with the keyboard selects too.

use bevy::prelude::*;

use super::edit::Motion;
use super::keys::{Held, current, keep};
use super::layout::Labels;
use super::{Cursor, TextBox, TextFocus};
use crate::common::{TouchGesture, UiPointer, in_interface};
use crate::widgets::ButtonPressed;

/// Clicks this close in time and place count as one double or triple click.
const MULTI_CLICK_SECONDS: f32 = 0.4;
const MULTI_CLICK_PIXELS: f32 = 6.0;
/// How far the pointer must move, held down, before it is a drag.
const DRAG_PIXELS: f32 = 4.0;

/// The click being followed.
#[derive(Default)]
pub(in crate::widgets) struct Clicking {
    /// The box pressed, where and when, and which click in a row it was.
    last: Option<(Entity, Vec2, f32, u8)>,
    /// The box pressed and held, where the press was, and the place a drag
    /// from there selects from.
    dragging: Option<(Entity, Vec2, usize)>,
}

/// How many clicks in a row this one makes: one, two or three, then one again.
fn clicks_in_a_row(last: Option<(Entity, Vec2, f32, u8)>, at: (Entity, Vec2, f32)) -> u8 {
    match last {
        Some((entity, place, when, count))
            if entity == at.0
                && place.distance(at.1) <= MULTI_CLICK_PIXELS
                && at.2 - when <= MULTI_CLICK_SECONDS =>
        {
            count % 3 + 1
        }
        _ => 1,
    }
}

/// Presses on boxes, and the pointer held down after one. Before `take_text`,
/// which gives the box the keyboard, so a box getting it can be told apart.
#[allow(clippy::too_many_arguments)]
pub(in crate::widgets) fn select_by_pointer(
    mut pressed: MessageReader<ButtonPressed>,
    focus: Res<TextFocus>,
    mut boxes: Query<(&mut TextBox, &mut Cursor)>,
    labels: Labels,
    pointer: Res<UiPointer>,
    mouse: Res<ButtonInput<MouseButton>>,
    held: Res<Held>,
    time: Res<Time<Real>>,
    mut clicking: Local<Clicking>,
) {
    let now = time.elapsed_secs();
    let on_box = pressed
        .read()
        .map(|press| press.0)
        .filter(|&entity| boxes.contains(entity))
        .last();
    if let (Some(entity), Some(at)) = (on_box, pointer.0)
        && let Ok((mut text_box, mut cursor)) = boxes.get_mut(entity)
    {
        let mut edited = current(&text_box, &cursor);
        let place = labels.place_at(entity, edited.len(), at.x);
        let count = clicks_in_a_row(clicking.last, (entity, at, now));
        clicking.last = Some((entity, at, now, count));
        let getting_keyboard = focus.0 != Some(entity);
        let anchor = if getting_keyboard && edited.selection().is_some() {
            // Only takes the keyboard, keeping what was selected; a second
            // click (or a drag) goes on as usual.
            clicking.last = None;
            place
        } else {
            match count {
                1 => edited.move_caret(Motion::To(place), held.shift()),
                2 => edited.select_word(place),
                _ => edited.select_all(),
            }
            edited.anchor
        };
        clicking.dragging = (count == 1).then_some((entity, at, anchor));
        keep(edited, &mut text_box, &mut cursor, now);
        return;
    }
    if !mouse.pressed(MouseButton::Left) {
        clicking.dragging = None;
        return;
    }
    let (Some((entity, from, anchor)), Some(at)) = (clicking.dragging, pointer.0) else {
        return;
    };
    if from.distance(at) <= DRAG_PIXELS {
        return;
    }
    if let Ok((mut text_box, mut cursor)) = boxes.get_mut(entity) {
        let mut edited = current(&text_box, &cursor);
        edited.anchor = anchor.min(edited.len());
        edited.caret = labels.place_at(entity, edited.len(), at.x);
        keep(edited, &mut text_box, &mut cursor, now);
    }
}

/// A finger sliding over the box with the keyboard selects from where it
/// landed, rather than scrolling or moving what the box is on. Before
/// `scroll_areas` and `take_presses`, which would take the drag.
#[allow(clippy::too_many_arguments)]
pub(in crate::widgets) fn drag_to_select(
    focus: Res<TextFocus>,
    mut gesture: ResMut<TouchGesture>,
    pointer: Res<UiPointer>,
    ui_scale: Res<UiScale>,
    mut boxes: Query<(&mut TextBox, &mut Cursor, &ComputedNode, &UiGlobalTransform)>,
    labels: Labels,
    time: Res<Time<Real>>,
    mut started: Local<Option<(Vec2, usize)>>,
) {
    let (Some(entity), Some(at)) = (focus.0, pointer.0) else {
        return;
    };
    if gesture.drag == Vec2::ZERO {
        *started = None;
        return;
    }
    let Ok((mut text_box, mut cursor, node, transform)) = boxes.get_mut(entity) else {
        return;
    };
    let start = in_interface(gesture.drag_start, &ui_scale);
    if !node.contains_point(*transform, start / node.inverse_scale_factor) {
        return;
    }
    let mut edited = current(&text_box, &cursor);
    let anchor = match *started {
        Some((from, anchor)) if from == gesture.drag_start => anchor,
        _ => {
            let anchor = labels.place_at(entity, edited.len(), start.x);
            *started = Some((gesture.drag_start, anchor));
            anchor
        }
    };
    edited.anchor = anchor;
    edited.caret = labels.place_at(entity, edited.len(), at.x);
    keep(edited, &mut text_box, &mut cursor, time.elapsed_secs());
    *gesture = TouchGesture::default();
}

#[cfg(test)]
mod tests;
