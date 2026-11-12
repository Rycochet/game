use bevy::ecs::message::MessageCursor;
use bevy::input::ButtonState;
use bevy::input::keyboard::{Key, KeyboardInput, NativeKey, NativeKeyCode};
use bevy::input::mouse::{MouseButtonInput, MouseWheel};
use bevy::input::touch::{TouchInput, TouchPhase};
use bevy::window::PrimaryWindow;

use super::keys::{logical_key, typed_key};
use super::mouse::drag_steps;
use super::*;
use crate::dev_tools::FakeCursor;

#[test]
fn keys_mean_what_they_type() {
    assert_eq!(logical_key(KeyCode::KeyM), Key::Character("m".into()));
    assert_eq!(logical_key(KeyCode::Digit3), Key::Character("3".into()));
    assert_eq!(logical_key(KeyCode::Escape), Key::Escape);
    assert_eq!(
        logical_key(KeyCode::F1),
        Key::Unidentified(NativeKey::Unidentified)
    );
    assert_eq!(typed_key('Q'), (KeyCode::KeyQ, Key::Character("Q".into())));
    assert_eq!(
        typed_key('7'),
        (KeyCode::Digit7, Key::Character("7".into()))
    );
    assert_eq!(typed_key(' '), (KeyCode::Space, Key::Space));
    assert_eq!(
        typed_key('!'),
        (
            KeyCode::Unidentified(NativeKeyCode::Unidentified),
            Key::Character("!".into())
        )
    );
}

#[test]
fn a_drag_presses_moves_and_lets_go() {
    let from = Vec2::new(0.0, 0.0);
    let to = Vec2::new(10.0, 0.0);
    let steps = drag_steps(from, to, 2, MouseButton::Left, false);
    assert_eq!(
        steps,
        [
            Step::MoveTo(from),
            Step::Wait,
            Step::Mouse(MouseButton::Left, ButtonState::Pressed),
            Step::MoveTo(Vec2::new(5.0, 0.0)),
            Step::MoveTo(to),
            Step::Mouse(MouseButton::Left, ButtonState::Released),
            Step::Wait,
        ]
    );
    let steps = drag_steps(from, to, 1, MouseButton::Left, true);
    assert_eq!(
        steps,
        [
            Step::Finger(TouchPhase::Started, from),
            Step::Finger(TouchPhase::Moved, to),
            Step::Finger(TouchPhase::Ended, to),
            Step::Wait,
        ]
    );
}

/// An app playing fake input as the game does, without a real window.
fn input_app() -> App {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .add_message::<MouseButtonInput>()
        .add_message::<MouseWheel>()
        .add_message::<TouchInput>()
        .add_message::<KeyboardInput>()
        .init_resource::<FakeInput>()
        .init_resource::<FakeCursor>()
        .add_systems(PreUpdate, (play, ignore_real_mouse).chain());
    app.world_mut().spawn((Window::default(), PrimaryWindow));
    app
}

#[test]
fn steps_play_one_a_frame() {
    let mut app = input_app();
    app.world_mut().resource_mut::<FakeInput>().steps.extend([
        Step::Wait,
        Step::Mouse(MouseButton::Left, ButtonState::Pressed),
    ]);
    app.update();
    assert_eq!(app.world().resource::<FakeInput>().steps.len(), 1);
    app.update();
    let fake = app.world().resource::<FakeInput>();
    assert!(fake.steps.is_empty());
    assert_eq!(fake.frame, 2);
    let written = app.world().resource::<Messages<MouseButtonInput>>();
    assert_eq!(written.len(), 1);
}

#[test]
fn moving_holds_the_fake_cursor_not_the_window_s() {
    let mut app = input_app();
    let at = Vec2::new(30.0, 20.0);
    app.world_mut()
        .resource_mut::<FakeInput>()
        .steps
        .extend([Step::MoveTo(at), Step::LetGo]);
    app.update();
    assert_eq!(app.world().resource::<FakeCursor>().at, Some(at));
    let mut windows = app.world_mut().query::<&Window>();
    let window = windows.single(app.world()).unwrap();
    assert_eq!(window.cursor_position(), None);
    app.update();
    assert_eq!(app.world().resource::<FakeCursor>().at, None);
}

/// A press or release of the left button, as the real mouse sends them.
fn real(app: &mut App, state: ButtonState) {
    let mut windows = app
        .world_mut()
        .query_filtered::<Entity, With<PrimaryWindow>>();
    let window = windows.single(app.world()).unwrap();
    app.world_mut().write_message(MouseButtonInput {
        button: MouseButton::Left,
        state,
        window,
    });
}

/// The left button's presses and releases a reader sees, after a frame.
fn read_states(app: &mut App, reader: &mut MessageCursor<MouseButtonInput>) -> Vec<ButtonState> {
    app.update();
    let messages = app.world().resource::<Messages<MouseButtonInput>>();
    reader.read(messages).map(|press| press.state).collect()
}

#[test]
fn real_presses_are_ignored_while_the_fake_cursor_is_held() {
    let mut app = input_app();
    let mut reader = MessageCursor::default();
    // Not held: the real mouse works.
    real(&mut app, ButtonState::Pressed);
    assert_eq!(read_states(&mut app, &mut reader), [ButtonState::Pressed]);
    // Held: a real press is dropped, a real release kept, a fake press kept.
    app.world_mut().resource_mut::<FakeCursor>().at = Some(Vec2::ONE);
    real(&mut app, ButtonState::Pressed);
    real(&mut app, ButtonState::Released);
    app.world_mut()
        .resource_mut::<FakeInput>()
        .steps
        .push_back(Step::Mouse(MouseButton::Left, ButtonState::Pressed));
    assert_eq!(
        read_states(&mut app, &mut reader),
        [ButtonState::Released, ButtonState::Pressed]
    );
    // Nothing is seen twice.
    assert_eq!(read_states(&mut app, &mut reader), []);
}
