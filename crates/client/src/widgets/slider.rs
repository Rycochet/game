//! A slider: a track, often a picture of what its values mean, with a mark along
//! it. By mouse it is pressed and dragged; by touch tapped or dragged.

use bevy::prelude::*;
use bevy::ui::UiGlobalTransform;

use super::theme::SUNKEN_BACKGROUND;
use super::{ButtonPressed, INK, Pressable, RAISED_BACKGROUND, raised_border};
use crate::common::{TouchGesture, UiPointer, in_interface};
use crate::dom::Finder;

const HEIGHT: f32 = 18.0;
const BORDER: f32 = 1.0;
const MARK_WIDTH: f32 = 12.0;
const MARK_EDGE: f32 = 2.0;

/// Where a slider's mark is, `0..=1` along it. Set it to move the mark.
#[derive(Component, Clone, Copy, Debug, PartialEq)]
pub struct Slider(pub f32);

/// The player moved this slider to `.1`. Setting `Slider` doesn't send this.
#[derive(Message, Clone, Copy, Debug, PartialEq)]
pub struct SliderMoved(pub Entity, pub f32);

/// The mark on the slider `.0`.
#[derive(Component)]
pub(super) struct SliderMark(Entity);

/// The slider the mouse is holding.
#[derive(Resource, Default)]
pub(super) struct HeldSlider(Option<Entity>);

/// A slider `width` across with its mark at `value`, showing `track` stretched
/// along it, or plain with `None`. Add a marker to it to tell which it is in
/// `SliderMoved`.
pub fn spawn_slider<'a>(
    parent: &'a mut ChildSpawnerCommands,
    value: f32,
    track: Option<Handle<Image>>,
    width: f32,
) -> EntityCommands<'a> {
    let mut slider = parent.spawn((
        Slider(value),
        Pressable,
        Node {
            width: px(width),
            height: px(HEIGHT),
            flex_shrink: 0.0,
            border: UiRect::all(px(BORDER)),
            ..default()
        },
        BorderColor::from(INK),
    ));
    match track {
        Some(track) => slider.insert(ImageNode::new(track).with_mode(NodeImageMode::Stretch)),
        None => slider.insert(BackgroundColor(SUNKEN_BACKGROUND)),
    };
    let id = slider.id();
    // The mark's centre runs along a rail half a mark in from each end, so the
    // whole mark stays inside the frame.
    slider.with_children(|slider| {
        slider
            .spawn(Node {
                position_type: PositionType::Absolute,
                left: px(MARK_WIDTH / 2.0),
                right: px(MARK_WIDTH / 2.0),
                height: percent(100),
                ..default()
            })
            .with_child((
                SliderMark(id),
                Node {
                    position_type: PositionType::Absolute,
                    left: percent(value * 100.0),
                    width: px(MARK_WIDTH),
                    height: percent(100),
                    margin: UiRect::left(px(-MARK_WIDTH / 2.0)),
                    border: UiRect::all(px(MARK_EDGE)),
                    ..default()
                },
                BackgroundColor(RAISED_BACKGROUND),
                raised_border(),
            ));
    });
    slider
}

/// How far along a track `left` to `left + width` a point is.
fn value_at(x: f32, left: f32, width: f32) -> f32 {
    ((x - left) / width.max(1.0)).clamp(0.0, 1.0)
}

/// How far along a slider's node a point in the interface's pixels is: along
/// the rail its mark's centre runs on, inside the border and half a mark in.
fn along(node: &ComputedNode, transform: &UiGlobalTransform, at: Vec2) -> f32 {
    let scale = node.inverse_scale_factor;
    let size = node.size() * scale;
    let centre = transform.translation * scale;
    let inset = BORDER + MARK_WIDTH / 2.0;
    value_at(at.x, centre.x - size.x / 2.0 + inset, size.x - inset * 2.0)
}

/// Moves a slider to where the pointer is, if that is a move.
fn move_slider(
    entity: Entity,
    at: Vec2,
    sliders: &mut Query<(
        &mut Slider,
        &ComputedNode,
        &UiGlobalTransform,
        &InheritedVisibility,
    )>,
    moved: &mut MessageWriter<SliderMoved>,
) {
    if let Ok((mut slider, node, transform, _)) = sliders.get_mut(entity) {
        let value = along(node, transform, at);
        if slider.0 != value {
            slider.0 = value;
            moved.write(SliderMoved(entity, value));
        }
    }
}

/// A press on a slider takes it (and moves the mark there), until let go.
pub(super) fn hold_sliders(
    mouse: Res<ButtonInput<MouseButton>>,
    pointer: Res<UiPointer>,
    mut pressed: MessageReader<ButtonPressed>,
    mut held: ResMut<HeldSlider>,
    mut sliders: Query<(
        &mut Slider,
        &ComputedNode,
        &UiGlobalTransform,
        &InheritedVisibility,
    )>,
    mut moved: MessageWriter<SliderMoved>,
) {
    for press in pressed.read() {
        if sliders.contains(press.0) {
            held.0 = Some(press.0);
            // A tap may be over by the next frame.
            if let Some(at) = pointer.0 {
                move_slider(press.0, at, &mut sliders, &mut moved);
            }
        }
    }
    if !mouse.pressed(MouseButton::Left) {
        held.0 = None;
    }
    if let (Some(entity), Some(at)) = (held.0, pointer.0) {
        move_slider(entity, at, &mut sliders, &mut moved);
    }
}

/// A finger dragging from a slider moves it. Before `take_presses`, which takes
/// drags starting on panels.
pub(super) fn drag_sliders(
    pointer: Res<UiPointer>,
    gesture: Res<TouchGesture>,
    ui_scale: Res<UiScale>,
    mut sliders: Query<(
        &mut Slider,
        &ComputedNode,
        &UiGlobalTransform,
        &InheritedVisibility,
    )>,
    finder: Finder,
    mut moved: MessageWriter<SliderMoved>,
) {
    let Some(at) = pointer.0.filter(|_| gesture.drag != Vec2::ZERO) else {
        return;
    };
    // The slider the finger landed on, if it is what was in front there.
    let chain = finder.chain_at(in_interface(gesture.drag_start, &ui_scale));
    if let Some(entity) = chain.into_iter().find(|&entity| sliders.contains(entity)) {
        move_slider(entity, at, &mut sliders, &mut moved);
    }
}

/// Puts the marks where their sliders say.
pub(super) fn place_marks(sliders: Query<&Slider>, mut marks: Query<(&SliderMark, &mut Node)>) {
    for (mark, mut node) in &mut marks {
        if let Ok(slider) = sliders.get(mark.0) {
            let left = percent(slider.0 * 100.0);
            if node.left != left {
                node.left = left;
            }
        }
    }
}

#[cfg(test)]
mod tests;
