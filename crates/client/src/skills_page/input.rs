//! The page's input, after the widgets have looked for presses and before the
//! game, while the Skills tab is on show. Only over the tree (and in front there:
//! a panel over it keeps its presses): dragging moves the tree, the wheel or a
//! pinch zooms about the pointer or the fingers, a click or tap chooses a skill
//! (or nothing), the arrow keys move it, + and - zoom and Home fits it.

use bevy::ecs::system::SystemParam;
use bevy::input::mouse::{AccumulatedMouseScroll, MouseScrollUnit};
use bevy::prelude::*;
use bevy::ui::UiGlobalTransform;

use super::build::SkillNode;
use super::spawn::{Canvas, PageButton};
use super::view::{SkillsView, ZOOM_STEP};
use crate::common::{Action, Preferences, TouchGesture, UiPointer, in_interface};
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
/// The keys the page takes while the pointer is over the tree.
const PAGE_KEYS: [KeyCode; 9] = [
    KeyCode::ArrowLeft,
    KeyCode::ArrowRight,
    KeyCode::ArrowUp,
    KeyCode::ArrowDown,
    KeyCode::Equal,
    KeyCode::NumpadAdd,
    KeyCode::Minus,
    KeyCode::NumpadSubtract,
    KeyCode::Home,
];

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

/// The canvas's corner, size and middle.
type Area = (Vec2, Vec2, Vec2);

/// The canvas, if on show.
type CanvasParts<'a> = (
    &'a ComputedNode,
    &'a UiGlobalTransform,
    &'a InheritedVisibility,
);

pub(super) fn take_input(
    mut inputs: Inputs,
    others: Others,
    buttons: Query<&PageButton>,
    canvas: Query<CanvasParts, With<Canvas>>,
    nodes: Query<&SkillNode>,
    mut view: ResMut<SkillsView>,
) {
    let presses: Vec<Entity> = inputs.pressed.read().map(|press| press.0).collect();
    // A hidden tab has no size.
    let area = canvas.iter().find_map(|(node, transform, visible)| {
        let (corner, size) = logical_rect(node, transform);
        (visible.get() && size.min_element() > 0.0).then_some((corner, size, corner + size / 2.0))
    });
    let Some((corner, size, middle)) = area else {
        view.shown = false;
        view.pressed_at = None;
        view.drag_from = None;
        view.hovered = None;
        return;
    };
    if !view.shown {
        view.fit_wanted = true;
    }
    view.shown = true;
    for &press in &presses {
        match buttons.get(press) {
            Ok(PageButton::ZoomIn) => view.zoom_about(ZOOM_STEP, Vec2::ZERO),
            Ok(PageButton::ZoomOut) => view.zoom_about(1.0 / ZOOM_STEP, Vec2::ZERO),
            Ok(PageButton::Fit) => view.fit_wanted = true,
            Err(_) => {}
        }
    }
    if view.fit_wanted && !nodes.is_empty() {
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
    if others.any_open() {
        return;
    }
    let over = point_and_drag(&mut inputs, &mut view, &nodes, (corner, size, middle));
    if over {
        keys_move(&mut inputs, &mut view);
        // These are the page's here, so nothing behind it moves.
        for key in PAGE_KEYS {
            inputs.keys.reset(key);
        }
        inputs.scroll.delta = Vec2::ZERO;
    }
}

/// The mouse and fingers: dragging, zooming and choosing. Whether the pointer is
/// over the tree.
fn point_and_drag(
    inputs: &mut Inputs,
    view: &mut SkillsView,
    nodes: &Query<&SkillNode>,
    (corner, size, middle): Area,
) -> bool {
    let within = |at: Vec2| {
        at.x >= corner.x
            && at.y >= corner.y
            && at.x <= corner.x + size.x
            && at.y <= corner.y + size.y
    };
    let pointer = inputs.pointer.0;
    let inside = pointer.filter(|&at| within(at));
    // A press still here was on the canvas, in front: panels keep their own.
    if let Some(at) = inside
        && inputs.mouse.just_pressed(MouseButton::Left)
    {
        inputs.mouse.clear_just_pressed(MouseButton::Left);
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
    if let Some(at) = inside
        && notches != 0.0
    {
        view.zoom_about(ZOOM_STEP.powf(notches), at - middle);
    }
    let touch = *inputs.gesture;
    let centre = in_interface(touch.centre, &inputs.ui_scale);
    if touch.pinch != 1.0 && within(centre) {
        view.zoom_about(touch.pinch, centre - middle);
        *inputs.gesture = TouchGesture::default();
    } else if touch.drag != Vec2::ZERO && within(in_interface(touch.drag_start, &inputs.ui_scale)) {
        view.pan += in_interface(touch.drag, &inputs.ui_scale);
        *inputs.gesture = TouchGesture::default();
    }
    // What is under the pointer shows in the line below, while not dragging.
    let dragging = view
        .pressed_at
        .is_some_and(|from| inside.is_some_and(|at| from.distance(at) > CLICK_SLOP));
    view.hovered = inside
        .filter(|_| !dragging)
        .and_then(|at| chosen_at(view, nodes, at - middle));
    inside.is_some()
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
