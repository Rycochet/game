//! Moving the dots and turning the triangles: a press on a triangle holds its dot
//! (it jumps there), a press beside them turns them all, following the mouse or
//! finger until let go. Keys: 1, 2 and 3 choose a triangle, the arrows nudge its
//! dot, Q and E turn.

use bevy::prelude::*;
use bevy::ui::UiGlobalTransform;
use shared::sheet::Row;

use super::draft::Draft;
use super::geometry::{angle_round_middle, corners, dot_at, dot_place, near_triangle, turned_by};
use super::screen::Chooser;
use crate::common::{TouchGesture, UiPointer, in_interface};
use crate::widgets::{ButtonPressed, TextFocus};

/// How near a triangle a press still takes its dot, in the chooser's units.
const DOT_MARGIN: f32 = 0.03;
/// How far a nudge moves a dot, in the chooser's units.
const NUDGE: f32 = 0.004;
/// How far Q and E turn, in degrees a frame.
const TURN_PER_FRAME: f32 = 1.0;

/// What a press on the chooser is doing, until let go.
#[derive(Resource, Default, Clone, Copy, Debug, PartialEq)]
pub(super) enum Holding {
    #[default]
    Nothing,
    Dot(Row),
    /// Turning, from where the press began.
    Turn {
        from_angle: f32,
        from_turn: u16,
    },
}

/// Where a point in the interface's pixels is in the chooser's units.
fn in_chooser(at: Vec2, node: &ComputedNode, transform: &UiGlobalTransform) -> Vec2 {
    let scale = node.inverse_scale_factor;
    let size = node.size() * scale;
    let corner = transform.translation * scale - size / 2.0;
    (at - corner) / size.max(Vec2::ONE)
}

/// What a press here starts.
fn press_at(point: Vec2, draft: &Draft) -> Holding {
    let turn = draft.build.turn;
    match Row::ALL
        .into_iter()
        .find(|&row| near_triangle(point, corners(row, turn), DOT_MARGIN))
    {
        Some(row) => Holding::Dot(row),
        None => Holding::Turn {
            from_angle: angle_round_middle(point),
            from_turn: turn,
        },
    }
}

/// What holding does with the pointer here.
fn follow(holding: Holding, point: Vec2, draft: &mut Draft) {
    match holding {
        Holding::Nothing => {}
        Holding::Dot(row) => {
            draft.chosen = row;
            let dot = dot_at(point, corners(row, draft.build.turn));
            draft.build.dots[row as usize] = dot;
        }
        Holding::Turn {
            from_angle,
            from_turn,
        } => {
            let moved = angle_round_middle(point) - from_angle;
            draft.build.turn = turned_by(from_turn, moved);
        }
    }
}

#[allow(clippy::too_many_arguments)]
pub(super) fn point_at_triangles(
    mut pressed: MessageReader<ButtonPressed>,
    mouse: Res<ButtonInput<MouseButton>>,
    pointer: Res<UiPointer>,
    gesture: Res<TouchGesture>,
    ui_scale: Res<UiScale>,
    chooser: Single<(Entity, &ComputedNode, &UiGlobalTransform), With<Chooser>>,
    mut holding: ResMut<Holding>,
    mut draft: ResMut<Draft>,
) {
    let (entity, node, transform) = *chooser;
    let pressed_here = pressed.read().any(|press| press.0 == entity);
    // A finger sliding from the chooser holds it too: panels take slides.
    let start = in_interface(gesture.drag_start, &ui_scale);
    let slid_here = gesture.drag != Vec2::ZERO
        && *holding == Holding::Nothing
        && node.contains_point(*transform, start / node.inverse_scale_factor);
    if pressed_here || slid_here {
        let from = if slid_here {
            start
        } else {
            pointer.0.unwrap_or(start)
        };
        *holding = press_at(in_chooser(from, node, transform), &draft);
    }
    if let (Some(at), true) = (pointer.0, *holding != Holding::Nothing) {
        follow(*holding, in_chooser(at, node, transform), &mut draft);
    }
    if !mouse.pressed(MouseButton::Left) && gesture.first_finger.is_none() {
        *holding = Holding::Nothing;
    }
}

/// The keys, unless a box has them.
pub(super) fn keys_for_triangles(
    keys: Res<ButtonInput<KeyCode>>,
    focus: Res<TextFocus>,
    mut draft: ResMut<Draft>,
) {
    if focus.0.is_some() {
        return;
    }
    for (key, row) in [
        (KeyCode::Digit1, Row::Body),
        (KeyCode::Digit2, Row::Mind),
        (KeyCode::Digit3, Row::Spirit),
    ] {
        if keys.just_pressed(key) {
            draft.chosen = row;
        }
    }
    let turning = keys.pressed(KeyCode::KeyE) as i32 - keys.pressed(KeyCode::KeyQ) as i32;
    if turning != 0 {
        draft.build.turn = turned_by(draft.build.turn, turning as f32 * TURN_PER_FRAME);
    }
    let nudge = Vec2::new(
        keys.pressed(KeyCode::ArrowRight) as i32 as f32
            - keys.pressed(KeyCode::ArrowLeft) as i32 as f32,
        keys.pressed(KeyCode::ArrowDown) as i32 as f32
            - keys.pressed(KeyCode::ArrowUp) as i32 as f32,
    );
    if nudge != Vec2::ZERO {
        let row = draft.chosen;
        let moved = dot_place(row, &draft.build) + nudge * NUDGE;
        let dot = dot_at(moved, corners(row, draft.build.turn));
        draft.build.dots[row as usize] = dot;
    }
}

#[cfg(test)]
mod tests;
