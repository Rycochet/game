//! The page's input, after the widgets have looked for presses on it and before
//! the game: K (Skills) opens and closes it; K, Esc, its close button or a press
//! outside it closes it. While it is open the mouse, fingers and keys are its:
//! dragging moves the tree, the wheel or a pinch zooms about the pointer or the
//! fingers, a click or tap chooses a skill (or nothing), the arrow keys move it,
//! + and - zoom and Home fits it to the page.

use bevy::ecs::system::SystemParam;
use bevy::input::mouse::{AccumulatedMouseScroll, MouseScrollUnit};
use bevy::prelude::*;
use bevy::ui::UiGlobalTransform;

use super::build::SkillNode;
use super::spawn::{Canvas, PageButton, SkillsLayer};
use super::view::{SkillsView, ZOOM_STEP};
use crate::common::{Action, Preferences, SkillsPageOpen, TouchGesture, UiPointer, in_interface};
use crate::debug_menu::DebugMenu;
use crate::minimap::FullMap;
use crate::system_menu::SystemMenu;
use crate::widgets::{ButtonPressed, logical_rect};

/// A press and let go within this far of each other is a click, not a drag.
const CLICK_SLOP: f32 = 6.0;
/// How far round a node a click still chooses it, for fingers.
const NODE_SLOP: f32 = 8.0;
/// How fast the arrow keys move the tree, in interface pixels a second.
const KEY_PAN_SPEED: f32 = 600.0;
/// Wheel movement in pixels that counts as one notch.
const PIXELS_PER_NOTCH: f32 = 40.0;

/// What else is open, which the page leaves the input to.
#[derive(SystemParam)]
pub(super) struct Others<'w> {
    debug_menu: Res<'w, DebugMenu>,
    full_map: Res<'w, FullMap>,
    system_menu: Res<'w, SystemMenu>,
}

impl Others<'_> {
    fn any_open(&self) -> bool {
        self.debug_menu.open || self.full_map.open || self.system_menu.is_open()
    }
}

/// The input the page takes.
#[derive(SystemParam)]
pub(super) struct Inputs<'w, 's> {
    keys: ResMut<'w, ButtonInput<KeyCode>>,
    mouse: ResMut<'w, ButtonInput<MouseButton>>,
    scroll: ResMut<'w, AccumulatedMouseScroll>,
    gesture: ResMut<'w, TouchGesture>,
    pressed: MessageReader<'w, 's, ButtonPressed>,
    pointer: Res<'w, UiPointer>,
    ui_scale: Res<'w, UiScale>,
    preferences: Res<'w, Preferences>,
    time: Res<'w, Time<Real>>,
}

#[allow(clippy::too_many_arguments)]
pub(super) fn take_input(
    mut inputs: Inputs,
    others: Others,
    buttons: Query<&PageButton>,
    mut layer: Single<(Entity, &mut Visibility), With<SkillsLayer>>,
    canvas: Single<(&ComputedNode, &UiGlobalTransform), With<Canvas>>,
    nodes: Query<&SkillNode>,
    mut open: ResMut<SkillsPageOpen>,
    mut view: ResMut<SkillsView>,
) {
    let presses: Vec<Entity> = inputs.pressed.read().map(|press| press.0).collect();
    if others.any_open() {
        return;
    }
    let controls = &inputs.preferences.controls;
    let skills_key = controls.just_pressed(&inputs.keys, Action::Skills);
    let was_open = *layer.1 == Visibility::Visible;
    if skills_key && !was_open {
        open.0 = true;
    }
    if open.0 && !was_open {
        view.fit_wanted = true;
    }
    if open.0 && was_open {
        let (layer_entity, _) = *layer;
        let closes = presses
            .iter()
            .any(|&entity| entity == layer_entity || buttons.get(entity) == Ok(&PageButton::Close));
        if skills_key || closes || inputs.keys.just_pressed(KeyCode::Escape) {
            open.0 = false;
        }
    }
    let shown = if open.0 {
        Visibility::Visible
    } else {
        Visibility::Hidden
    };
    layer.1.set_if_neq(shown);
    if !open.0 {
        view.pressed_at = None;
        view.drag_from = None;
        return;
    }

    let (corner, size) = logical_rect(canvas.0, canvas.1);
    let middle = corner + size / 2.0;
    for &press in &presses {
        match buttons.get(press) {
            Ok(PageButton::ZoomIn) => view.zoom_about(ZOOM_STEP, Vec2::ZERO),
            Ok(PageButton::ZoomOut) => view.zoom_about(1.0 / ZOOM_STEP, Vec2::ZERO),
            Ok(PageButton::Fit) => view.fit_wanted = true,
            _ => {}
        }
    }
    if view.fit_wanted && !nodes.is_empty() && size.min_element() > 0.0 {
        // The whole tree, circles included, at zoom 1.
        let extent = nodes.iter().fold(
            (Vec2::splat(f32::MAX), Vec2::splat(f32::MIN)),
            |(low, high), node| {
                let half = Vec2::splat(node.size / 2.0);
                (low.min(node.at - half), high.max(node.at + half))
            },
        );
        view.fit(size, extent);
        view.fit_wanted = false;
    }
    point_and_drag(&mut inputs, &mut view, &nodes, (corner, size, middle));
    keys_move(&mut inputs, &mut view);

    // The rest is the page's, so nothing behind it moves.
    inputs.keys.reset_all();
    inputs.mouse.clear_just_pressed(MouseButton::Left);
    inputs.mouse.clear_just_released(MouseButton::Left);
    inputs.mouse.reset(MouseButton::Right);
    inputs.scroll.delta = Vec2::ZERO;
    *inputs.gesture = TouchGesture::default();
}

/// The mouse and fingers: dragging, zooming and choosing.
fn point_and_drag(
    inputs: &mut Inputs,
    view: &mut SkillsView,
    nodes: &Query<&SkillNode>,
    (corner, size, middle): (Vec2, Vec2, Vec2),
) {
    let pointer = inputs.pointer.0;
    let inside = pointer.filter(|at| {
        at.x >= corner.x
            && at.y >= corner.y
            && at.x <= corner.x + size.x
            && at.y <= corner.y + size.y
    });
    if let Some(at) = inside
        && inputs.mouse.just_pressed(MouseButton::Left)
    {
        view.pressed_at = Some(at);
        view.drag_from = Some(at);
    }
    if let (Some(from), Some(at)) = (view.drag_from, pointer)
        && inputs.mouse.pressed(MouseButton::Left)
    {
        view.pan += at - from;
        view.drag_from = Some(at);
    }
    if !inputs.mouse.pressed(MouseButton::Left) || inputs.mouse.just_released(MouseButton::Left) {
        if let (Some(from), Some(at)) = (view.pressed_at, pointer)
            && from.distance(at) <= CLICK_SLOP
        {
            view.chosen = chosen_at(view, nodes, at - middle);
        }
        view.pressed_at = None;
        view.drag_from = None;
    }
    let notches = match inputs.scroll.unit {
        MouseScrollUnit::Line => inputs.scroll.delta.y,
        MouseScrollUnit::Pixel => inputs.scroll.delta.y / PIXELS_PER_NOTCH,
    };
    if notches != 0.0 {
        let about = inside.map_or(Vec2::ZERO, |at| at - middle);
        view.zoom_about(ZOOM_STEP.powf(notches), about);
    }
    let touch = *inputs.gesture;
    if touch.pinch != 1.0 {
        let about = in_interface(touch.centre, &inputs.ui_scale) - middle;
        view.zoom_about(touch.pinch, about);
    }
    if touch.drag != Vec2::ZERO {
        view.pan += in_interface(touch.drag, &inputs.ui_scale);
    }
    // What is under the pointer shows in the line below, while not dragging.
    let dragging = view
        .pressed_at
        .is_some_and(|from| inside.is_some_and(|at| from.distance(at) > CLICK_SLOP));
    view.hovered = inside
        .filter(|_| !dragging)
        .and_then(|at| chosen_at(view, nodes, at - middle));
}

/// The skill whose node is nearest `at` (from the middle of the page), if `at`
/// is on or near one.
fn chosen_at(
    view: &SkillsView,
    nodes: &Query<&SkillNode>,
    at: Vec2,
) -> Option<shared::skill::SkillId> {
    nodes
        .iter()
        .filter_map(|node| {
            let distance = view.on_page(node.at).distance(at);
            (distance <= node.size * view.zoom / 2.0 + NODE_SLOP).then_some((node, distance))
        })
        .min_by(|a, b| a.1.total_cmp(&b.1))
        .and_then(|(node, _)| node.id)
}

/// The arrow keys move the tree, + and - zoom, Home fits it.
fn keys_move(inputs: &mut Inputs, view: &mut SkillsView) {
    let keys = &inputs.keys;
    let mut direction = Vec2::ZERO;
    for (key, step) in [
        (KeyCode::ArrowLeft, Vec2::X),
        (KeyCode::ArrowRight, Vec2::NEG_X),
        (KeyCode::ArrowUp, Vec2::Y),
        (KeyCode::ArrowDown, Vec2::NEG_Y),
    ] {
        if keys.pressed(key) {
            direction += step;
        }
    }
    view.pan += direction * KEY_PAN_SPEED * inputs.time.delta_secs();
    if keys.any_just_pressed([KeyCode::Equal, KeyCode::NumpadAdd]) {
        view.zoom_about(ZOOM_STEP, Vec2::ZERO);
    }
    if keys.any_just_pressed([KeyCode::Minus, KeyCode::NumpadSubtract]) {
        view.zoom_about(1.0 / ZOOM_STEP, Vec2::ZERO);
    }
    let controls = &inputs.preferences.controls;
    if keys.just_pressed(KeyCode::Home) || controls.just_pressed(keys, Action::ResetView) {
        view.fit_wanted = true;
    }
}
