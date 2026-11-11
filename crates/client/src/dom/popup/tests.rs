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

#[test]
fn a_popup_is_placed_and_closes_on_escape_unless_locked() {
    let mut app = built(&POPUP);
    let popup = find(&mut app, "a/box");
    let node = app.world().get::<Node>(popup).unwrap();
    assert_eq!((node.left, node.top), (px(40), px(60)));
    assert_eq!(node.position_type, PositionType::Absolute);

    assert_eq!(escape(&mut app), vec![UiId::new("a", "box")]);

    // Locked: it stays.
    app.world_mut().get_mut::<Popup>(popup).unwrap().locked = true;
    assert!(escape(&mut app).is_empty());
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
