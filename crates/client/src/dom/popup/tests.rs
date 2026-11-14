use bevy::ecs::system::RunSystemOnce;

use super::*;
use crate::dom::build::tests::{built, find};
use crate::dom::{Template, UiEvent, UiValue};
use crate::html;

static POPUP: Template = html!(
    "popup",
    r#"<form id="a"><popup id="box" left="40" top="60"><p id="words">Hello</p></popup></form>"#
);

/// A frame with Esc just pressed: who was told they closed.
fn escape(app: &mut App) -> Vec<UiId> {
    let mut keys = app.world_mut().resource_mut::<ButtonInput<KeyCode>>();
    keys.release(KeyCode::Escape);
    keys.clear();
    keys.press(KeyCode::Escape);
    app.world_mut().run_system_once(close_popups).unwrap();
    let mut events = app.world_mut().resource_mut::<Messages<UiEvent>>();
    let closed = events
        .drain()
        .filter(|event| event.value == UiValue::Closed);
    closed.map(|event| event.id).collect()
}

/// A frame with the left button just pressed away from everything.
fn press_elsewhere(app: &mut App) -> Vec<UiId> {
    app.world_mut().resource_mut::<UiPointer>().0 = Some(Vec2::splat(900.0));
    let mut mouse = app.world_mut().resource_mut::<ButtonInput<MouseButton>>();
    mouse.release(MouseButton::Left);
    mouse.clear();
    mouse.press(MouseButton::Left);
    app.world_mut().run_system_once(close_popups).unwrap();
    let mut events = app.world_mut().resource_mut::<Messages<UiEvent>>();
    let closed = events
        .drain()
        .filter(|event| event.value == UiValue::Closed);
    closed.map(|event| event.id).collect()
}

#[test]
fn a_popup_is_placed_and_closes_by_itself_unless_locked() {
    let mut app = built(&POPUP);
    let popup = find(&mut app, "a/box");
    let node = app.world().get::<Node>(popup).unwrap();
    assert_eq!((node.left, node.top), (px(40), px(60)));
    assert_eq!(node.position_type, PositionType::Absolute);
    let id = UiId::new("a", "box");

    assert_eq!(press_elsewhere(&mut app), vec![id]);
    assert_eq!(escape(&mut app), vec![id]);

    // Locked: it stays through a press elsewhere, and Esc still closes it.
    app.world_mut().get_mut::<Popup>(popup).unwrap().locked = true;
    assert!(press_elsewhere(&mut app).is_empty());
    assert_eq!(escape(&mut app), vec![id]);
}

#[test]
fn a_popup_is_taken_hold_of_by_its_top_only() {
    let (corner, size) = (Vec2::new(100.0, 100.0), Vec2::new(200.0, 80.0));
    assert!(drag::on_grip(Vec2::new(150.0, 110.0), corner, size));
    assert!(!drag::on_grip(
        Vec2::new(150.0, 100.0 + drag::GRIP + 1.0),
        corner,
        size
    ));
    assert!(!drag::on_grip(Vec2::new(90.0, 110.0), corner, size));
}

#[test]
fn a_press_on_its_padlock_locks_it_and_another_unlocks() {
    let mut app = built(&POPUP);
    let popup = find(&mut app, "a/box");
    let mut locks = app.world_mut().query::<(Entity, &PopupLock)>();
    let (lock, _) = locks.iter(app.world()).next().expect("a padlock");
    for locked in [true, false] {
        // (Only this press: the system is run afresh each time.)
        let mut presses = app.world_mut().resource_mut::<Messages<ButtonPressed>>();
        presses.clear();
        app.world_mut().write_message(ButtonPressed(lock));
        app.world_mut().run_system_once(press_locks).unwrap();
        assert_eq!(app.world().get::<Popup>(popup).unwrap().locked, locked);
    }
}

static HOVER: Template = html!(
    "popup.hover",
    r#"<form id="h"><popup id="box" hover left="40" top="60"><p>About it</p></popup></form>"#
);

/// A frame of hover popups closing by themselves, with the pointer at `at`,
/// and this one kept or not: who was told they closed.
fn hover_frame(app: &mut App, at: Option<Vec2>, kept: bool) -> Vec<UiId> {
    let id = UiId::new("h", "box");
    app.world_mut().resource_mut::<UiPointer>().0 = at;
    app.world_mut().resource_mut::<KeptOpen>().0 = if kept { vec![id] } else { vec![] };
    app.world_mut()
        .run_system_once(hover::close_hovers)
        .unwrap();
    app.world_mut().flush();
    let mut events = app.world_mut().resource_mut::<Messages<UiEvent>>();
    let closed = events
        .drain()
        .filter(|event| event.value == UiValue::Closed);
    closed.map(|event| event.id).collect()
}

/// As though the pointer last rested on it or its subject a while ago.
fn long_ago(app: &mut App, popup: Entity) {
    let ago = hover::LastKept(-1.0 - hover::GRACE);
    app.world_mut().entity_mut(popup).insert(ago);
}

#[test]
fn a_hover_popup_closes_once_the_pointer_leaves_it_and_its_subject() {
    let mut app = built(&HOVER);
    app.init_resource::<KeptOpen>();
    app.init_resource::<Time<Real>>();
    let popup = find(&mut app, "h/box");
    assert!(app.world().get::<Popup>(popup).unwrap().hover);
    let away = Some(Vec2::splat(900.0));
    // Newly shown: it waits.
    assert!(hover_frame(&mut app, away, false).is_empty());
    // On its subject: it stays.
    long_ago(&mut app, popup);
    assert!(hover_frame(&mut app, away, true).is_empty());
    // No pointer (a finger lifted): it stays.
    long_ago(&mut app, popup);
    assert!(hover_frame(&mut app, None, false).is_empty());
    // Locked: it stays.
    long_ago(&mut app, popup);
    app.world_mut().get_mut::<Popup>(popup).unwrap().locked = true;
    assert!(hover_frame(&mut app, away, false).is_empty());
    // Left a while ago, unlocked: it goes.
    app.world_mut().get_mut::<Popup>(popup).unwrap().locked = false;
    assert_eq!(
        hover_frame(&mut app, away, false),
        vec![UiId::new("h", "box")]
    );
}
