//! Dragging and raising panels, and their close buttons.

use bevy::prelude::*;
use bevy::ui::{CalculatedClip, ComputedStackIndex, UiGlobalTransform};

use super::super::item::shown_at;
use super::super::{ButtonPressed, ButtonState, Pressable, RaisePanel};
use super::{FIRST_PANEL_Z, MoveablePanel, PanelPart};
use crate::common::{TouchGesture, UiPointer, in_interface};

/// What is being moved, between frames.
#[derive(Resource, Default)]
pub(in crate::widgets) struct Dragging {
    /// By the mouse: which panel.
    mouse: Option<Entity>,
    /// The last place the pointer was, while dragging by the mouse.
    from: Option<Vec2>,
    /// Where the finger now dragging landed, and the panel its title bar was
    /// there, if it was.
    touch: Option<(Vec2, Option<Entity>)>,
    /// The highest panel so far.
    top_z: i32,
}

/// What this takes from the mouse: a press on a widget, and it is gone, so
/// nothing under the widget sees it. A tap arrives as a press (see `touch`), so
/// buttons work by touch; moving a panel by touch comes from `TouchGesture`.
#[allow(clippy::too_many_arguments, clippy::type_complexity)]
pub(in crate::widgets) fn take_presses(
    mut commands: Commands,
    mut mouse: ResMut<ButtonInput<MouseButton>>,
    pointer: Res<UiPointer>,
    mut gesture: ResMut<TouchGesture>,
    scale: Res<UiScale>,
    parts: Query<(
        &ComputedStackIndex,
        &ComputedNode,
        &UiGlobalTransform,
        &InheritedVisibility,
        &PanelPart,
    )>,
    buttons: Query<
        (
            &ComputedStackIndex,
            &ComputedNode,
            &UiGlobalTransform,
            &InheritedVisibility,
            Entity,
            Option<&ButtonState>,
            Option<&CalculatedClip>,
        ),
        With<Pressable>,
    >,
    mut raised: MessageReader<RaisePanel>,
    mut panels: Query<(&mut MoveablePanel, &mut GlobalZIndex)>,
    mut pressed: MessageWriter<ButtonPressed>,
    mut dragging: ResMut<Dragging>,
) {
    let dragging = &mut *dragging;
    // The part in front at a point, and how far in front it is.
    let stacked_part_at = |at: Vec2| {
        parts
            .iter()
            .filter(|(_, node, transform, visible, ..)| {
                visible.get() && node.contains_point(**transform, at / node.inverse_scale_factor)
            })
            .max_by_key(|(stack, ..)| stack.0)
            .map(|(stack, .., part)| (stack.0, *part))
    };
    let part_at = |at: Vec2| stacked_part_at(at).map(|(_, part)| part);
    let mut raise = |panel: Entity, panels: &mut Query<(&mut MoveablePanel, &mut GlobalZIndex)>| {
        if let Ok((_, mut z)) = panels.get_mut(panel) {
            dragging.top_z = dragging.top_z.max(FIRST_PANEL_Z) + 1;
            z.0 = dragging.top_z;
        }
    };
    for panel in raised.read() {
        raise(panel.0, &mut panels);
    }

    if mouse.just_pressed(MouseButton::Left) {
        let button = pointer.0.and_then(|at| {
            buttons
                .iter()
                .filter(|(_, node, transform, visible, .., clip)| {
                    visible.get() && shown_at(at, node, transform, *clip)
                })
                .max_by_key(|(stack, ..)| stack.0)
        });
        let stacked = pointer.0.and_then(stacked_part_at);
        // Only what is in front: a button hidden by another panel gets nothing.
        let button = button.filter(|(stack, ..)| stacked.is_none_or(|(part, _)| stack.0 > part));
        let part = stacked.map(|(_, part)| part);
        if let Some((.., entity, state, _)) = button {
            mouse.clear_just_pressed(MouseButton::Left);
            if !state.is_some_and(|state| state.is_disabled()) {
                pressed.write(ButtonPressed(entity));
            }
        } else if part.is_some_and(|part| !matches!(part, PanelPart::LeftForOwner(_))) {
            mouse.clear_just_pressed(MouseButton::Left);
        }
        // A press on a button in a panel is a press on the panel, but its
        // title bar and close button don't have buttons on them.
        match part {
            Some(PanelPart::Close(panel)) => {
                commands.entity(panel).despawn();
            }
            Some(PanelPart::TitleBar(panel)) => {
                raise(panel, &mut panels);
                dragging.mouse = Some(panel);
                dragging.from = pointer.0;
            }
            Some(PanelPart::Body(panel)) => raise(panel, &mut panels),
            Some(PanelPart::LeftForOwner(_)) | None => {}
        }
    }
    if !mouse.pressed(MouseButton::Left) {
        dragging.mouse = None;
    }
    if let (Some(panel), Some(from), Some(to)) = (dragging.mouse, dragging.from, pointer.0) {
        dragging.from = Some(to);
        if let Ok((mut moveable, _)) = panels.get_mut(panel) {
            moveable.at += to - from;
        }
    }

    if gesture.drag != Vec2::ZERO {
        let start = in_interface(gesture.drag_start, &scale);
        if dragging.touch.is_none_or(|(known, _)| known != start) {
            dragging.touch = Some((
                start,
                match part_at(start) {
                    Some(PanelPart::TitleBar(panel)) => Some(panel),
                    Some(PanelPart::LeftForOwner(_)) | None => None,
                    Some(_) => Some(Entity::PLACEHOLDER),
                },
            ));
        }
        if let Some((_, Some(panel))) = dragging.touch {
            if let Ok((mut moveable, _)) = panels.get_mut(panel) {
                moveable.at += in_interface(gesture.drag, &scale);
            }
            gesture.drag = Vec2::ZERO;
        }
    }
}
