//! The caret: a thin line where typing goes, in the box with the keyboard. It
//! stays solid while the player types or moves it, then fades gently in and
//! out, twice a second (solid throughout with Reduce Motion). Long writing is
//! slid left to keep it in sight.

use std::f32::consts::TAU;

use bevy::prelude::*;

use super::keys::current;
use super::layout::Labels;
use super::{BORDER, Cursor, SIDE_PADDING, TextBox, TextBoxCaret, TextBoxLabel, TextFocus};
use crate::common::Preferences;
use crate::widgets::INK;

/// How wide the caret is, in interface pixels.
pub(super) const WIDTH: f32 = 2.0;
/// How long it stays solid after moving, in seconds.
const SOLID_SECONDS: f32 = 0.5;
/// How many times a second it fades in and out after that.
const PULSES_PER_SECOND: f32 = 2.0;

/// How strongly the caret shows, `seconds` after it last moved.
pub(super) fn strength(seconds: f32, reduce_motion: bool) -> f32 {
    if reduce_motion || seconds < SOLID_SECONDS {
        return 1.0;
    }
    let turn = (seconds - SOLID_SECONDS) * PULSES_PER_SECOND;
    0.5 + 0.5 * (turn * TAU).cos()
}

/// How far to slide writing `width` wide across a gap `room` wide, from
/// `scroll`, to keep the place `at` in sight with the caret's width.
pub(super) fn slide(scroll: f32, at: f32, width: f32, room: f32) -> f32 {
    let room = (room - WIDTH).max(0.0);
    let mut scroll = scroll;
    if at - scroll > room {
        scroll = at - room;
    }
    if at < scroll {
        scroll = at;
    }
    // No empty room after the end while there is writing before.
    scroll.min((width - room).max(0.0)).max(0.0)
}

/// Places each box's caret and slides its writing. After `show_text_boxes`.
#[allow(clippy::too_many_arguments, clippy::type_complexity)]
pub(in crate::widgets) fn show_carets(
    focus: Res<TextFocus>,
    preferences: Res<Preferences>,
    time: Res<Time<Real>>,
    mut boxes: Query<(&TextBox, &mut Cursor, &ComputedNode)>,
    labels: Labels,
    mut label_nodes: Query<(&TextBoxLabel, &mut Node), Without<TextBoxCaret>>,
    mut carets: Query<(
        &TextBoxCaret,
        &mut Node,
        &mut BackgroundColor,
        &mut Visibility,
        &ComputedNode,
    )>,
    mut had_keyboard: Local<Option<Entity>>,
) {
    let now = time.elapsed_secs();
    if *had_keyboard != focus.0 {
        // A box getting the keyboard starts with the caret solid.
        if let Some((_, mut cursor, _)) = focus.0.and_then(|entity| boxes.get_mut(entity).ok()) {
            cursor.moved_at = now;
        }
        *had_keyboard = focus.0;
    }
    for (caret, mut node, mut colour, mut visibility, caret_node) in &mut carets {
        let Ok((text_box, mut cursor, box_node)) = boxes.get_mut(caret.0) else {
            continue;
        };
        let edited = current(text_box, &cursor);
        let Some((_, edges)) = labels.places(caret.0, edited.len()) else {
            continue;
        };
        let scale = box_node.inverse_scale_factor;
        let inside = box_node.size() * scale - Vec2::splat(BORDER * 2.0);
        let at = edges[edited.caret.min(edges.len() - 1)];
        let width = *edges.last().expect("at least one place") + WIDTH;
        let scroll = slide(cursor.scroll, at, width, inside.x - SIDE_PADDING * 2.0);
        if (scroll - cursor.scroll).abs() > 0.01 {
            cursor.scroll = scroll;
        }
        for (label, mut label_node) in &mut label_nodes {
            if label.0 == caret.0 && label_node.left != px(SIDE_PADDING - scroll) {
                label_node.left = px(SIDE_PADDING - scroll);
            }
        }
        let place = (
            px(SIDE_PADDING + at - scroll),
            px((inside.y - caret_node.size().y * caret_node.inverse_scale_factor) / 2.0),
        );
        if (node.left, node.top) != place {
            (node.left, node.top) = place;
        }
        let shows = focus.0 == Some(caret.0);
        let wanted = if shows {
            Visibility::Inherited
        } else {
            Visibility::Hidden
        };
        visibility.set_if_neq(wanted);
        let alpha = strength(now - cursor.moved_at, preferences.graphics.reduce_motion);
        colour.set_if_neq(BackgroundColor(INK.with_alpha(alpha)));
    }
}

#[cfg(test)]
mod tests;
