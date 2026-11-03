//! Fake keys: presses and typing, and what each key means.

use bevy::input::ButtonState;
use bevy::input::keyboard::{Key, NativeKey, NativeKeyCode};
use bevy::prelude::*;
use serde::Deserialize;
use server::debug_api::{BrpResult, Value, json, params, refused};

use super::{FakeInput, Step, one, queued};

#[derive(Deserialize)]
pub(super) struct KeyParams {
    /// Bevy's `KeyCode` name: `Escape`, `Backquote`, `KeyM`, `Enter`,
    /// `ArrowUp`, `Digit1`, `F1`, ...
    key: KeyCode,
    #[serde(default = "one")]
    hold_frames: u32,
    /// Keys held down round it, such as `["ControlLeft"]` for Ctrl+V.
    #[serde(default)]
    with: Vec<KeyCode>,
}

/// `client.key { "key": "Escape" }`: presses and lets go of a key (holding it
/// `hold_frames`), with any keys in `with` held down from a frame before.
pub(super) fn key(In(request): In<Option<Value>>, mut fake: ResMut<FakeInput>) -> BrpResult {
    let KeyParams {
        key,
        hold_frames,
        with,
    } = params(request)?;
    let held = |code: KeyCode, state| Step::Key {
        code,
        logical: logical_key(code),
        state,
    };
    let mut steps: Vec<Step> = with
        .iter()
        .map(|&code| held(code, ButtonState::Pressed))
        .collect();
    if !with.is_empty() {
        steps.push(Step::Wait);
    }
    steps.push(held(key, ButtonState::Pressed));
    steps.extend((1..hold_frames).map(|_| Step::Wait));
    steps.push(held(key, ButtonState::Released));
    steps.extend(with.iter().map(|&code| held(code, ButtonState::Released)));
    steps.push(Step::Wait);
    queued(&mut fake, steps)
}

#[derive(Deserialize)]
pub(super) struct TypeParams {
    text: String,
}

/// `client.type { "text": "castle" }`: types it, a key a frame, into
/// whatever has the keyboard (a text box with focus, say).
pub(super) fn type_text(In(request): In<Option<Value>>, mut fake: ResMut<FakeInput>) -> BrpResult {
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

/// What a key means when pressed alone: the letter or digit it types, or its
/// name.
pub(super) fn logical_key(code: KeyCode) -> Key {
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
pub(super) fn typed_key(c: char) -> (KeyCode, Key) {
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
