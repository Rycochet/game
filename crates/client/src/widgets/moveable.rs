//! Panels that fit their contents and can be moved by their title bars, and
//! the taking of clicks and touches for widgets.

use bevy::prelude::*;
use bevy::ui::{CalculatedClip, ComputedStackIndex, UiGlobalTransform};
use bevy::window::PrimaryWindow;

use super::item::shown_at;
use super::{
    ButtonPressed, ButtonState, PANEL_EDGE, Pressable, RaisePanel, SHADED_EDGE, panel, panel_title,
    spawn_close_button,
};
use crate::common::{TouchGesture, UiPointer, in_interface, interface_area};

/// Above the maps, below full-screen layers; a panel clicked on goes above the
/// others.
const FIRST_PANEL_Z: i32 = 30;

const PADDING: f32 = 8.0;
const CONTENT_GAP: f32 = 4.0;
const TITLE_GAP: f32 = 8.0;
/// The stripes either side of the title, which only panels that can be moved have:
/// how many, how thick, the gap between, and the least each side may shrink to.
const STRIPES: usize = 3;
const STRIPE_THICKNESS: f32 = 2.0;
const STRIPE_GAP: f32 = 4.0;
const MIN_STRIPES_WIDTH: f32 = 16.0;
/// Round the close button, still on it, for fingers.
const CLOSE_MARGIN: f32 = 4.0;

/// A panel that can be moved. `at` is where its top-left corner is wanted, in
/// logical pixels; it is kept on the screen.
#[derive(Component, Clone, Copy, Debug)]
pub(super) struct MoveablePanel {
    at: Vec2,
    /// The window's size when last placed, to keep it in the same part of the
    /// window when that changes.
    window: Option<Vec2>,
}

/// What a click on this node does.
#[derive(Component, Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum PanelPart {
    /// Only stops the click going through the panel.
    Body(Entity),
    TitleBar(Entity),
    Close(Entity),
    /// Stops it reaching widgets under it, but leaves it for its owner to read
    /// from the mouse and fingers.
    LeftForOwner(Entity),
}

/// A moveable panel with this title, at this place. Returns the panel, and the
/// entity to put its contents in; the panel fits them. It is despawned when
/// closed.
pub fn spawn_moveable_panel(commands: &mut Commands, title: &str, at: Vec2) -> (Entity, Entity) {
    let root = commands
        .spawn((
            panel(),
            MoveablePanel { at, window: None },
            GlobalZIndex(FIRST_PANEL_Z),
            Visibility::Hidden,
        ))
        .id();
    commands.entity(root).insert((
        PanelPart::Body(root),
        Node {
            position_type: PositionType::Absolute,
            flex_direction: FlexDirection::Column,
            border: UiRect::all(px(PANEL_EDGE)),
            padding: UiRect::all(px(PADDING)),
            row_gap: px(CONTENT_GAP),
            ..default()
        },
    ));
    let content = commands
        .spawn(Node {
            flex_direction: FlexDirection::Column,
            row_gap: px(CONTENT_GAP),
            ..default()
        })
        .id();
    let bar = commands
        .spawn((
            PanelPart::TitleBar(root),
            Node {
                align_items: AlignItems::Center,
                column_gap: px(TITLE_GAP),
                ..default()
            },
        ))
        .with_children(|bar| {
            spawn_stripes(bar);
            bar.spawn(panel_title(title));
            spawn_stripes(bar);
            bar.spawn((
                PanelPart::Close(root),
                Node {
                    padding: UiRect::all(px(CLOSE_MARGIN)),
                    ..default()
                },
            ))
            .with_children(|holder| {
                spawn_close_button(holder, default());
            });
        })
        .id();
    commands.entity(root).add_children(&[bar, content]);
    // In front of any panels already open.
    commands.write_message(RaisePanel(root));
    (root, content)
}

/// Put this on a node for presses on it to go no further. `entity` is the node.
pub fn stops_presses(entity: Entity) -> impl Bundle {
    PanelPart::Body(entity)
}

/// Lines running from the title out to the edge of the panel.
fn spawn_stripes(bar: &mut ChildSpawnerCommands) {
    bar.spawn(Node {
        flex_grow: 1.0,
        min_width: px(MIN_STRIPES_WIDTH),
        flex_direction: FlexDirection::Column,
        row_gap: px(STRIPE_GAP),
        ..default()
    })
    .with_children(|stripes| {
        for _ in 0..STRIPES {
            stripes.spawn((
                Node {
                    height: px(STRIPE_THICKNESS),
                    ..default()
                },
                BackgroundColor(SHADED_EDGE),
            ));
        }
    });
}

/// Where a panel of this size goes, wanted at `at`: wholly on the screen if it
/// fits, otherwise with its top left on the screen, for the title bar.
fn on_screen(at: Vec2, size: Vec2, window: Vec2) -> Vec2 {
    at.min(window - size).max(Vec2::ZERO)
}

/// Where something at `at` belongs once the window has changed size: the same
/// share of the way across it.
fn in_new_window(at: Vec2, old: Vec2, new: Vec2) -> Vec2 {
    at / old * new
}

/// Puts panels where they are wanted, as far as the window allows.
pub(super) fn place_panels(
    window: Single<&Window, With<PrimaryWindow>>,
    scale: Res<UiScale>,
    mut panels: Query<(
        &mut MoveablePanel,
        &ComputedNode,
        &mut Node,
        &mut Visibility,
    )>,
) {
    let window = interface_area(&window, &scale);
    // Minimised: nowhere to be, so everything stays as it is for coming back.
    if window.min_element() < 1.0 {
        return;
    }
    for (mut moveable, computed, mut node, mut visibility) in &mut panels {
        // Not laid out yet, so it can't be placed: it stays hidden until it can.
        if computed.size() == Vec2::ZERO {
            continue;
        }
        visibility.set_if_neq(Visibility::Inherited);
        if let Some(old) = moveable.window.replace(window) {
            moveable.at = in_new_window(moveable.at, old, window);
        }
        let size = computed.size() * computed.inverse_scale_factor;
        let at = on_screen(moveable.at, size, window);
        if moveable.at != at {
            moveable.at = at;
        }
        let (left, top) = (px(at.x), px(at.y));
        if node.left != left || node.top != top {
            node.left = left;
            node.top = top;
        }
    }
}

/// What is being moved, between frames.
#[derive(Resource, Default)]
pub(super) struct Dragging {
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
pub(super) fn take_presses(
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

#[cfg(test)]
mod tests;
