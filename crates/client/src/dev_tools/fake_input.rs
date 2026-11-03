//! Fake input for the debug API (only with the `debug-api` feature): moving
//! the pointer, clicking, dragging, scrolling, tapping, keys and typing, as
//! Bevy's own input messages, so the game can't tell them from a person.
//! Each request is queued, and one step is played per frame, so presses and
//! releases land on separate frames as they would by hand. See
//! `docs/context/debug-api.md`.

use std::collections::VecDeque;

use bevy::input::ButtonState;
use bevy::input::InputSystems;
use bevy::input::keyboard::{Key, KeyboardInput, NativeKey, NativeKeyCode};
use bevy::input::mouse::{MouseButtonInput, MouseScrollUnit, MouseWheel};
use bevy::input::touch::{TouchInput, TouchPhase};
use bevy::prelude::*;
use bevy::window::PrimaryWindow;
use serde::Deserialize;
use server::debug_api::{AddDebugMethod, BrpResult, Value, json, params, refused};

/// The id fake fingers use, well away from real ones.
const FINGER: u64 = 9_000_000;

pub(super) fn add(app: &mut App) {
    app.init_resource::<FakeInput>()
        .add_systems(PreUpdate, play.before(InputSystems))
        .add_debug_method("client.move", move_to)
        .add_debug_method("client.click", click)
        .add_debug_method("client.drag", drag)
        .add_debug_method("client.scroll", scroll)
        .add_debug_method("client.key", key)
        .add_debug_method("client.type", type_text)
        .add_debug_method("client.wait", wait)
        .add_debug_method("client.input", input);
}

/// One step of fake input, played in one frame.
#[derive(Clone, Debug, PartialEq)]
enum Step {
    /// The mouse cursor goes here (logical pixels).
    MoveTo(Vec2),
    Mouse(MouseButton, ButtonState),
    /// Lines of the mouse wheel; positive scrolls up (zooms in).
    Wheel(f32),
    /// A finger, at a point.
    Finger(TouchPhase, Vec2),
    Key {
        code: KeyCode,
        logical: Key,
        state: ButtonState,
    },
    /// Nothing, for a frame.
    Wait,
}

/// What is still to be played, and how many frames have gone by.
#[derive(Resource, Default)]
struct FakeInput {
    steps: VecDeque<Step>,
    frame: u64,
}

/// Plays the next step, before Bevy reads this frame's input.
fn play(
    mut fake: ResMut<FakeInput>,
    mut window: Single<(Entity, &mut Window), With<PrimaryWindow>>,
    mut mouse: MessageWriter<MouseButtonInput>,
    mut wheel: MessageWriter<MouseWheel>,
    mut touch: MessageWriter<TouchInput>,
    mut keys: MessageWriter<KeyboardInput>,
) {
    fake.frame += 1;
    let Some(step) = fake.steps.pop_front() else {
        return;
    };
    let (entity, window) = &mut *window;
    let entity = *entity;
    match step {
        Step::MoveTo(at) => window.set_cursor_position(Some(at)),
        Step::Mouse(button, state) => {
            mouse.write(MouseButtonInput {
                button,
                state,
                window: entity,
            });
        }
        Step::Wheel(lines) => {
            wheel.write(MouseWheel {
                unit: MouseScrollUnit::Line,
                x: 0.0,
                y: lines,
                window: entity,
                phase: TouchPhase::Moved,
            });
        }
        Step::Finger(phase, position) => {
            touch.write(TouchInput {
                phase,
                position,
                window: entity,
                force: None,
                id: FINGER,
            });
        }
        Step::Key {
            code,
            logical,
            state,
        } => {
            let text = match (&logical, state) {
                (Key::Character(text), ButtonState::Pressed) => Some(text.clone()),
                (Key::Space, ButtonState::Pressed) => Some(" ".into()),
                _ => None,
            };
            keys.write(KeyboardInput {
                key_code: code,
                logical_key: logical,
                state,
                text,
                repeat: false,
                window: entity,
            });
        }
        Step::Wait => {}
    }
}

/// Queues `steps`, and answers so that `cargo xtask api` waits until they
/// have all been played.
fn queued(fake: &mut FakeInput, steps: impl IntoIterator<Item = Step>) -> BrpResult {
    fake.steps.extend(steps);
    Ok(json!({ "pending": fake.steps.len(), "wait_for_input": true }))
}

#[derive(Deserialize)]
struct At {
    x: f32,
    y: f32,
}

/// `client.move { "x": 400, "y": 300 }`: the mouse cursor goes there
/// (logical pixels), for what hovering shows.
fn move_to(In(request): In<Option<Value>>, mut fake: ResMut<FakeInput>) -> BrpResult {
    let At { x, y } = params(request)?;
    queued(&mut fake, [Step::MoveTo(Vec2::new(x, y))])
}

#[derive(Deserialize)]
struct ClickParams {
    x: f32,
    y: f32,
    /// `Left` (the default), `Right` or `Middle`.
    #[serde(default = "left")]
    button: MouseButton,
    /// Frames between pressing and letting go: long enough, a hold.
    #[serde(default = "one")]
    hold_frames: u32,
    /// A finger's tap instead of the mouse, through `touch`.
    #[serde(default)]
    touch: bool,
}

fn left() -> MouseButton {
    MouseButton::Left
}

fn one() -> u32 {
    1
}

/// `client.click { "x": 400, "y": 300 }`: moves there, presses and lets go
/// (`button`, `hold_frames`), or taps there with `"touch": true`.
fn click(In(request): In<Option<Value>>, mut fake: ResMut<FakeInput>) -> BrpResult {
    let ClickParams {
        x,
        y,
        button,
        hold_frames,
        touch,
    } = params(request)?;
    let at = Vec2::new(x, y);
    let held = (1..hold_frames).map(|_| Step::Wait);
    if touch {
        let steps = [Step::Finger(TouchPhase::Started, at)]
            .into_iter()
            .chain(held)
            .chain([Step::Finger(TouchPhase::Ended, at), Step::Wait]);
        queued(&mut fake, steps)
    } else {
        let steps = [
            Step::MoveTo(at),
            Step::Wait,
            Step::Mouse(button, ButtonState::Pressed),
        ]
        .into_iter()
        .chain(held)
        .chain([Step::Mouse(button, ButtonState::Released), Step::Wait]);
        queued(&mut fake, steps)
    }
}

#[derive(Deserialize)]
struct DragParams {
    from: [f32; 2],
    to: [f32; 2],
    /// Frames the move takes.
    #[serde(default = "ten")]
    steps: u32,
    #[serde(default = "left")]
    button: MouseButton,
    /// A finger instead of the mouse.
    #[serde(default)]
    touch: bool,
}

fn ten() -> u32 {
    10
}

/// `client.drag { "from": [100, 100], "to": [300, 200] }`: presses at
/// `from`, moves to `to` over `steps` frames and lets go; with
/// `"touch": true` a finger does it.
fn drag(In(request): In<Option<Value>>, mut fake: ResMut<FakeInput>) -> BrpResult {
    let DragParams {
        from,
        to,
        steps,
        button,
        touch,
    } = params(request)?;
    queued(
        &mut fake,
        drag_steps(Vec2::from(from), Vec2::from(to), steps, button, touch),
    )
}

/// The steps of a drag.
fn drag_steps(from: Vec2, to: Vec2, frames: u32, button: MouseButton, touch: bool) -> Vec<Step> {
    let frames = frames.max(1);
    let along = (1..=frames).map(|frame| from.lerp(to, frame as f32 / frames as f32));
    let mut steps = Vec::new();
    if touch {
        steps.push(Step::Finger(TouchPhase::Started, from));
        steps.extend(along.map(|at| Step::Finger(TouchPhase::Moved, at)));
        steps.push(Step::Finger(TouchPhase::Ended, to));
    } else {
        steps.extend([
            Step::MoveTo(from),
            Step::Wait,
            Step::Mouse(button, ButtonState::Pressed),
        ]);
        steps.extend(along.map(Step::MoveTo));
        steps.push(Step::Mouse(button, ButtonState::Released));
    }
    steps.push(Step::Wait);
    steps
}

#[derive(Deserialize)]
struct ScrollParams {
    /// Where the cursor is while scrolling, if not where it is now.
    x: Option<f32>,
    y: Option<f32>,
    /// Wheel lines: positive scrolls up (zooms in), negative down.
    lines: f32,
}

/// `client.scroll { "lines": 3 }`: the mouse wheel, at `x`, `y` if given.
fn scroll(In(request): In<Option<Value>>, mut fake: ResMut<FakeInput>) -> BrpResult {
    let ScrollParams { x, y, lines } = params(request)?;
    let mut steps: Vec<Step> = x
        .zip(y)
        .map(|(x, y)| Step::MoveTo(Vec2::new(x, y)))
        .into_iter()
        .collect();
    steps.extend([Step::Wheel(lines), Step::Wait]);
    queued(&mut fake, steps)
}

#[derive(Deserialize)]
struct KeyParams {
    /// Bevy's `KeyCode` name: `Escape`, `Backquote`, `KeyM`, `Enter`,
    /// `ArrowUp`, `Digit1`, `F1`, ...
    key: KeyCode,
    #[serde(default = "one")]
    hold_frames: u32,
}

/// `client.key { "key": "Escape" }`: presses and lets go of a key (holding it
/// `hold_frames`).
fn key(In(request): In<Option<Value>>, mut fake: ResMut<FakeInput>) -> BrpResult {
    let KeyParams { key, hold_frames } = params(request)?;
    let logical = logical_key(key);
    let steps = [Step::Key {
        code: key,
        logical: logical.clone(),
        state: ButtonState::Pressed,
    }]
    .into_iter()
    .chain((1..hold_frames).map(|_| Step::Wait))
    .chain([
        Step::Key {
            code: key,
            logical,
            state: ButtonState::Released,
        },
        Step::Wait,
    ]);
    queued(&mut fake, steps)
}

#[derive(Deserialize)]
struct TypeParams {
    text: String,
}

/// `client.type { "text": "castle" }`: types it, a key a frame, into
/// whatever has the keyboard (a text box with focus, say).
fn type_text(In(request): In<Option<Value>>, mut fake: ResMut<FakeInput>) -> BrpResult {
    let TypeParams { text } = params(request)?;
    if text.is_empty() {
        return Err(refused("nothing to type"));
    }
    let steps: Vec<Step> = text
        .chars()
        .flat_map(|c| {
            let (code, logical) = typed_key(c);
            [
                Step::Key {
                    code,
                    logical: logical.clone(),
                    state: ButtonState::Pressed,
                },
                Step::Key {
                    code,
                    logical,
                    state: ButtonState::Released,
                },
            ]
        })
        .chain([Step::Wait])
        .collect();
    queued(&mut fake, steps)
}

#[derive(Deserialize)]
struct WaitParams {
    #[serde(default = "one")]
    frames: u32,
}

/// `client.wait { "frames": 30 }`: answers once that many frames have gone
/// by after everything already queued (through `cargo xtask api`).
fn wait(In(request): In<Option<Value>>, mut fake: ResMut<FakeInput>) -> BrpResult {
    let WaitParams { frames } = params(request)?;
    let frames = frames.clamp(1, 100_000);
    queued(&mut fake, (0..frames).map(|_| Step::Wait))
}

/// `client.input`: how many steps of fake input are still to be played, and
/// the frame count, for waiting on them.
fn input(_: In<Option<Value>>, fake: Res<FakeInput>) -> BrpResult {
    Ok(json!({ "pending": fake.steps.len(), "frame": fake.frame }))
}

/// What a key means when pressed alone: the letter or digit it types, or its
/// name.
fn logical_key(code: KeyCode) -> Key {
    let character = |c: &str| Key::Character(c.into());
    match code {
        KeyCode::Escape => Key::Escape,
        KeyCode::Enter | KeyCode::NumpadEnter => Key::Enter,
        KeyCode::Backspace => Key::Backspace,
        KeyCode::Tab => Key::Tab,
        KeyCode::Space => Key::Space,
        KeyCode::Delete => Key::Delete,
        KeyCode::Home => Key::Home,
        KeyCode::End => Key::End,
        KeyCode::PageUp => Key::PageUp,
        KeyCode::PageDown => Key::PageDown,
        KeyCode::ArrowUp => Key::ArrowUp,
        KeyCode::ArrowDown => Key::ArrowDown,
        KeyCode::ArrowLeft => Key::ArrowLeft,
        KeyCode::ArrowRight => Key::ArrowRight,
        KeyCode::ShiftLeft | KeyCode::ShiftRight => Key::Shift,
        KeyCode::ControlLeft | KeyCode::ControlRight => Key::Control,
        KeyCode::AltLeft | KeyCode::AltRight => Key::Alt,
        KeyCode::Backquote => character("`"),
        KeyCode::Minus => character("-"),
        KeyCode::Equal => character("="),
        KeyCode::Comma => character(","),
        KeyCode::Period => character("."),
        KeyCode::Slash => character("/"),
        code => {
            let name = format!("{code:?}");
            match name
                .strip_prefix("Key")
                .or_else(|| name.strip_prefix("Digit"))
            {
                Some(rest) if rest.len() == 1 => character(&rest.to_lowercase()),
                _ => Key::Unidentified(NativeKey::Unidentified),
            }
        }
    }
}

/// The key that types `c`: letters and digits on their own keys, anything
/// else as text alone.
fn typed_key(c: char) -> (KeyCode, Key) {
    let code = match c.to_ascii_uppercase() {
        ' ' => return (KeyCode::Space, Key::Space),
        'A'..='Z' | '0'..='9' => {
            let name = if c.is_ascii_digit() {
                format!("Digit{c}")
            } else {
                format!("Key{}", c.to_ascii_uppercase())
            };
            params::<KeyCode>(Some(json!(name)))
                .unwrap_or(KeyCode::Unidentified(NativeKeyCode::Unidentified))
        }
        _ => KeyCode::Unidentified(NativeKeyCode::Unidentified),
    };
    (code, Key::Character(c.to_string().into()))
}

#[cfg(test)]
mod tests {
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
}
