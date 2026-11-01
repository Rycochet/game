//! A slider: a track, often a picture of what its values mean, with a mark along
//! it. By mouse it is pressed and dragged; by touch tapped or dragged.

use bevy::prelude::*;
use bevy::ui::UiGlobalTransform;

use super::fullscreen::Darkening;
use super::{ButtonPressed, INK, LIT_EDGE, Pressable};
use crate::common::{PointerPosition, TouchGesture};

const HEIGHT: f32 = 18.0;
const BORDER: f32 = 1.0;
const MARK_WIDTH: f32 = 4.0;
/// The mark's light edge, so it shows on a dark track as well as a light one.
const MARK_EDGE: f32 = 1.0;
const PLAIN_TRACK: Color = Color::srgb(0.75, 0.65, 0.48);

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
        None => slider.insert(BackgroundColor(PLAIN_TRACK)),
    };
    let id = slider.id();
    slider.with_child((
        SliderMark(id),
        Node {
            position_type: PositionType::Absolute,
            left: percent(value * 100.0),
            width: px(MARK_WIDTH),
            height: percent(100),
            margin: UiRect::left(px(-MARK_WIDTH / 2.0)),
            border: UiRect::horizontal(px(MARK_EDGE)),
            ..default()
        },
        BackgroundColor(INK),
        BorderColor::from(LIT_EDGE),
    ));
    slider
}

/// How far along a track `left` to `left + width` a point is.
fn value_at(x: f32, left: f32, width: f32) -> f32 {
    ((x - left) / width.max(1.0)).clamp(0.0, 1.0)
}

/// How far along a slider's node a point in logical pixels is.
fn along(node: &ComputedNode, transform: &UiGlobalTransform, at: Vec2) -> f32 {
    let scale = node.inverse_scale_factor;
    let size = node.size() * scale;
    let centre = transform.translation * scale;
    value_at(at.x, centre.x - size.x / 2.0, size.x)
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
    pointer: Res<PointerPosition>,
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
    pointer: Res<PointerPosition>,
    gesture: Res<TouchGesture>,
    mut sliders: Query<(
        &mut Slider,
        &ComputedNode,
        &UiGlobalTransform,
        &InheritedVisibility,
    )>,
    entities: Query<Entity, With<Slider>>,
    layers: Query<&InheritedVisibility, With<Darkening>>,
    mut moved: MessageWriter<SliderMoved>,
) {
    let Some(at) = pointer.0.filter(|_| gesture.drag != Vec2::ZERO) else {
        return;
    };
    // Sliders are all on moveable panels, under any full-screen layer.
    if layers.iter().any(|visible| visible.get()) {
        return;
    }
    for entity in &entities {
        let Ok((_, node, transform, visible)) = sliders.get(entity) else {
            continue;
        };
        let start = gesture.drag_start / node.inverse_scale_factor;
        if visible.get() && node.contains_point(*transform, start) {
            move_slider(entity, at, &mut sliders, &mut moved);
            return;
        }
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
mod tests {
    use bevy::ecs::system::RunSystemOnce;
    use bevy::math::Affine2;

    use super::*;

    /// A slider from (100, 50), 200 across; and the app it is in.
    fn test() -> (App, Entity) {
        let mut app = App::new();
        app.init_resource::<ButtonInput<MouseButton>>()
            .init_resource::<PointerPosition>()
            .init_resource::<TouchGesture>()
            .init_resource::<HeldSlider>()
            .add_message::<ButtonPressed>()
            .add_message::<SliderMoved>();
        let size = Vec2::new(200.0, 18.0);
        let slider = app
            .world_mut()
            .spawn((
                Slider(0.5),
                ComputedNode {
                    size,
                    inverse_scale_factor: 1.0,
                    ..default()
                },
                UiGlobalTransform::from(Affine2::from_translation(
                    Vec2::new(100.0, 50.0) + size / 2.0,
                )),
                InheritedVisibility::VISIBLE,
            ))
            .id();
        (app, slider)
    }

    fn value(app: &App, slider: Entity) -> f32 {
        app.world().entity(slider).get::<Slider>().unwrap().0
    }

    #[test]
    fn the_mouse_holds_a_slider_until_let_go() {
        let (mut app, slider) = test();
        let world = app.world_mut();
        world.resource_mut::<PointerPosition>().0 = Some(Vec2::new(150.0, 55.0));
        world
            .resource_mut::<ButtonInput<MouseButton>>()
            .press(MouseButton::Left);
        world.write_message(ButtonPressed(slider));
        world.run_system_once(hold_sliders).unwrap();
        assert_eq!(value(&app, slider), 0.25);
        // Each run here reads from the start, unlike in the game.
        let mut presses = app.world_mut().resource_mut::<Messages<ButtonPressed>>();
        presses.clear();

        // Off the slider, it still follows, as far as its end.
        let world = app.world_mut();
        world.resource_mut::<PointerPosition>().0 = Some(Vec2::new(900.0, 400.0));
        world.run_system_once(hold_sliders).unwrap();
        assert_eq!(value(&app, slider), 1.0);

        let world = app.world_mut();
        world
            .resource_mut::<ButtonInput<MouseButton>>()
            .release(MouseButton::Left);
        world.resource_mut::<PointerPosition>().0 = Some(Vec2::new(150.0, 55.0));
        world.run_system_once(hold_sliders).unwrap();
        assert_eq!(value(&app, slider), 1.0);
    }

    #[test]
    fn a_finger_drags_a_slider_it_landed_on() {
        let (mut app, slider) = test();
        let world = app.world_mut();
        world.resource_mut::<PointerPosition>().0 = Some(Vec2::new(150.0, 300.0));
        *world.resource_mut::<TouchGesture>() = TouchGesture {
            drag: Vec2::new(1.0, 0.0),
            drag_start: Vec2::new(200.0, 55.0),
            ..default()
        };
        world.run_system_once(drag_sliders).unwrap();
        assert_eq!(value(&app, slider), 0.25);

        // Landing elsewhere moves nothing.
        let world = app.world_mut();
        world.resource_mut::<TouchGesture>().drag_start = Vec2::new(200.0, 300.0);
        world.resource_mut::<PointerPosition>().0 = Some(Vec2::new(250.0, 55.0));
        world.run_system_once(drag_sliders).unwrap();
        assert_eq!(value(&app, slider), 0.25);
    }

    #[test]
    fn values_run_along_the_track_and_stop_at_its_ends() {
        assert_eq!(value_at(100.0, 100.0, 200.0), 0.0);
        assert_eq!(value_at(150.0, 100.0, 200.0), 0.25);
        assert_eq!(value_at(300.0, 100.0, 200.0), 1.0);
        assert_eq!(value_at(50.0, 100.0, 200.0), 0.0);
        assert_eq!(value_at(900.0, 100.0, 200.0), 1.0);
    }
}
