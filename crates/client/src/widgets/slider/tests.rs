use bevy::ecs::system::RunSystemOnce;
use bevy::math::Affine2;

use super::*;

/// A slider from (100, 50), 200 across; and the app it is in. Its mark's
/// centre runs from 107 to 293 (inside the border, half a mark in), so 153.5 is
/// a quarter of the way.
fn test() -> (App, Entity) {
    test_at(1.0)
}

/// The same, in the interface's pixels, with the interface drawn `scale` times
/// its size (in a window with no scaling of its own).
fn test_at(scale: f32) -> (App, Entity) {
    let mut app = App::new();
    app.init_resource::<ButtonInput<MouseButton>>()
        .init_resource::<UiPointer>()
        .insert_resource(UiScale(scale))
        .init_resource::<TouchGesture>()
        .init_resource::<HeldSlider>()
        .add_message::<ButtonPressed>()
        .add_message::<SliderMoved>();
    let size = Vec2::new(200.0, 18.0);
    let slider = app
        .world_mut()
        .spawn((
            Slider(0.5),
            // What the search for what a finger is on looks for.
            (Node::default(), crate::dom::Solid),
            ComputedNode {
                size: size * scale,
                inverse_scale_factor: 1.0 / scale,
                ..default()
            },
            UiGlobalTransform::from(Affine2::from_translation(
                (Vec2::new(100.0, 50.0) + size / 2.0) * scale,
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
    world.resource_mut::<UiPointer>().0 = Some(Vec2::new(153.5, 55.0));
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
    world.resource_mut::<UiPointer>().0 = Some(Vec2::new(900.0, 400.0));
    world.run_system_once(hold_sliders).unwrap();
    assert_eq!(value(&app, slider), 1.0);

    let world = app.world_mut();
    world
        .resource_mut::<ButtonInput<MouseButton>>()
        .release(MouseButton::Left);
    world.resource_mut::<UiPointer>().0 = Some(Vec2::new(153.5, 55.0));
    world.run_system_once(hold_sliders).unwrap();
    assert_eq!(value(&app, slider), 1.0);
}

#[test]
fn a_finger_drags_a_slider_it_landed_on() {
    let (mut app, slider) = test();
    let world = app.world_mut();
    world.resource_mut::<UiPointer>().0 = Some(Vec2::new(153.5, 300.0));
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
    world.resource_mut::<UiPointer>().0 = Some(Vec2::new(250.0, 55.0));
    world.run_system_once(drag_sliders).unwrap();
    assert_eq!(value(&app, slider), 0.25);
}

#[test]
fn a_finger_finds_its_slider_at_any_size() {
    for scale in [0.5, 2.0] {
        let (mut app, slider) = test_at(scale);
        let world = app.world_mut();
        world.resource_mut::<UiPointer>().0 = Some(Vec2::new(153.5, 300.0));
        // Where it landed is in window pixels.
        *world.resource_mut::<TouchGesture>() = TouchGesture {
            drag: Vec2::new(1.0, 0.0),
            drag_start: Vec2::new(200.0, 55.0) * scale,
            ..default()
        };
        world.run_system_once(drag_sliders).unwrap();
        assert_eq!(value(&app, slider), 0.25, "at {scale}");
    }
}

#[test]
fn the_mark_stops_half_a_mark_inside_the_frame() {
    let (mut app, slider) = test();
    let world = app.world_mut();
    world
        .resource_mut::<ButtonInput<MouseButton>>()
        .press(MouseButton::Left);
    world.write_message(ButtonPressed(slider));
    for (x, expected) in [
        (101.0, 0.0),
        (107.0, 0.0),
        (200.0, 0.5),
        (293.0, 1.0),
        (299.0, 1.0),
    ] {
        app.world_mut().resource_mut::<UiPointer>().0 = Some(Vec2::new(x, 55.0));
        app.world_mut().run_system_once(hold_sliders).unwrap();
        assert_eq!(value(&app, slider), expected, "at {x}");
    }
}

#[test]
fn values_run_along_the_track_and_stop_at_its_ends() {
    assert_eq!(value_at(100.0, 100.0, 200.0), 0.0);
    assert_eq!(value_at(150.0, 100.0, 200.0), 0.25);
    assert_eq!(value_at(300.0, 100.0, 200.0), 1.0);
    assert_eq!(value_at(50.0, 100.0, 200.0), 0.0);
    assert_eq!(value_at(900.0, 100.0, 200.0), 1.0);
}
