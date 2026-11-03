use bevy::input::ButtonState;
use bevy::input::keyboard::{Key, KeyboardInput, NativeKey, NativeKeyCode};
use bevy::input::mouse::{MouseButtonInput, MouseWheel};
use bevy::input::touch::{TouchInput, TouchPhase};
use bevy::window::PrimaryWindow;

use super::keys::{logical_key, typed_key};
use super::mouse::drag_steps;
use super::*;

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

#[test]
fn steps_play_one_a_frame() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .add_message::<MouseButtonInput>()
        .add_message::<MouseWheel>()
        .add_message::<TouchInput>()
        .add_message::<KeyboardInput>()
        .init_resource::<FakeInput>()
        .add_systems(PreUpdate, play);
    app.world_mut().spawn((Window::default(), PrimaryWindow));
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
