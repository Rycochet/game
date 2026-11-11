use bevy::input::mouse::AccumulatedMouseScroll;
use bevy::prelude::*;

use super::hit::tests::{element, laid};
use super::{Click, Hit, PointerEnter, PointerLeave, Press};
use crate::common::UiPointer;
use crate::dom::parts::{Hovered, Solid};

/// What the observers saw, in order.
#[derive(Resource, Default)]
struct Seen(Vec<(&'static str, Entity)>);

/// An app with the input systems, an element holding a smaller one, and a
/// plain solid node beside them. Every event is noted in `Seen`.
fn app() -> (App, Entity, Entity, Entity) {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .init_resource::<ButtonInput<MouseButton>>()
        .init_resource::<AccumulatedMouseScroll>()
        .init_resource::<UiPointer>()
        .init_resource::<Seen>();
    super::add(&mut app);
    app.add_observer(|event: On<Press>, mut seen: ResMut<Seen>| {
        seen.0.push(("press", event.entity));
    })
    .add_observer(|event: On<Click>, mut seen: ResMut<Seen>| {
        let name = if event.count == 2 { "double" } else { "click" };
        seen.0.push((name, event.entity));
    })
    .add_observer(|event: On<PointerEnter>, mut seen: ResMut<Seen>| {
        seen.0.push(("enter", event.entity));
    })
    .add_observer(|event: On<PointerLeave>, mut seen: ResMut<Seen>| {
        seen.0.push(("leave", event.entity));
    });
    let world = app.world_mut();
    let outer = (laid(Vec2::ZERO, Vec2::splat(100.0), 1), element());
    let outer = world.spawn(outer).id();
    let inner = (
        laid(Vec2::ZERO, Vec2::splat(20.0), 2),
        element(),
        ChildOf(outer),
    );
    let inner = world.spawn(inner).id();
    let beside = (laid(Vec2::new(200.0, 0.0), Vec2::splat(20.0), 3), Solid);
    let beside = world.spawn(beside).id();
    (app, outer, inner, beside)
}

/// A frame with the pointer at `at` and the left button as told.
fn frame(app: &mut App, at: Vec2, press: bool, release: bool) {
    app.world_mut().resource_mut::<UiPointer>().0 = Some(at);
    let mut mouse = app.world_mut().resource_mut::<ButtonInput<MouseButton>>();
    mouse.clear();
    if press {
        // Let go first, in case the last frame left it down.
        mouse.release(MouseButton::Left);
        mouse.clear();
        mouse.press(MouseButton::Left);
    }
    if release {
        mouse.release(MouseButton::Left);
    }
    app.update();
}

fn seen(app: &mut App) -> Vec<(&'static str, Entity)> {
    std::mem::take(&mut app.world_mut().resource_mut::<Seen>().0)
}

const ON_INNER: Vec2 = Vec2::splat(10.0);
const ON_OUTER: Vec2 = Vec2::splat(60.0);

#[test]
fn only_the_chain_is_hovered() {
    let (mut app, outer, inner, _) = app();
    frame(&mut app, ON_INNER, false, false);
    assert_eq!(seen(&mut app), [("enter", inner), ("enter", outer)]);
    assert!(app.world().get::<Hovered>(inner).is_some());
    assert_eq!(app.world().resource::<Hit>().chain, [inner, outer]);

    frame(&mut app, ON_OUTER, false, false);
    assert_eq!(seen(&mut app), [("leave", inner)]);
    assert!(app.world().get::<Hovered>(inner).is_none());
    assert!(app.world().get::<Hovered>(outer).is_some());
}

#[test]
fn events_go_to_the_target_and_up_through_what_holds_it() {
    let (mut app, outer, inner, _) = app();
    frame(&mut app, ON_INNER, true, false);
    let events = seen(&mut app);
    assert!(events.ends_with(&[("press", inner), ("press", outer)]));
    frame(&mut app, ON_INNER, false, true);
    assert_eq!(seen(&mut app), [("click", inner), ("click", outer)]);
}

#[test]
fn a_click_is_on_the_nearest_thing_pressed_and_let_go_on() {
    let (mut app, outer, _, _) = app();
    frame(&mut app, ON_INNER, true, false);
    seen(&mut app);
    // Let go off the inner element, but still on the outer one.
    frame(&mut app, ON_OUTER, false, true);
    assert_eq!(
        seen(&mut app),
        [("leave", app_inner(&app, outer)), ("click", outer)]
    );
}

/// The element inside `outer`.
fn app_inner(app: &App, outer: Entity) -> Entity {
    app.world().get::<Children>(outer).unwrap()[0]
}

#[test]
fn letting_go_somewhere_else_is_no_click() {
    let (mut app, ..) = app();
    frame(&mut app, ON_INNER, true, false);
    seen(&mut app);
    frame(&mut app, Vec2::new(210.0, 10.0), false, true);
    assert!(seen(&mut app).iter().all(|(name, _)| *name != "click"));
}

#[test]
fn a_second_click_soon_after_is_a_double_click() {
    let (mut app, _, inner, _) = app();
    for _ in 0..2 {
        frame(&mut app, ON_INNER, true, false);
        frame(&mut app, ON_INNER, false, true);
    }
    let clicks: Vec<_> = seen(&mut app)
        .into_iter()
        .filter(|(name, entity)| *entity == inner && *name != "press" && *name != "enter")
        .collect();
    assert_eq!(clicks, [("click", inner), ("double", inner)]);
}

#[test]
fn a_press_on_an_element_goes_no_further() {
    let (mut app, _, _, beside) = app();
    frame(&mut app, ON_INNER, true, false);
    let mouse = app.world().resource::<ButtonInput<MouseButton>>();
    assert!(!mouse.just_pressed(MouseButton::Left));

    // One on a node that isn't markup is left for whoever owns it.
    frame(&mut app, Vec2::new(210.0, 10.0), true, false);
    let mouse = app.world().resource::<ButtonInput<MouseButton>>();
    assert!(mouse.just_pressed(MouseButton::Left));
    assert_eq!(app.world().resource::<Hit>().target, Some(beside));
}
