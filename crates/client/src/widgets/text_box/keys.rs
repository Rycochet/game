//! Who has the keyboard, and what keys do to their box.

use bevy::input::ButtonState as KeyState;
use bevy::input::keyboard::{Key, KeyboardInput};
use bevy::prelude::*;

use super::edit::{Edited, Motion};
use super::{Cursor, PressedThisFrame, SoftKeyPressed, TextBox, TextEdited, TextFocus, allowed};
use crate::common::{Action, Preferences};
use crate::widgets::{ButtonPressed, clipboard, soft_keyboard};

/// The modifier keys held, followed from the key events themselves, as a box
/// with the keyboard clears the pressed keys every frame.
#[derive(Resource, Default)]
pub(in crate::widgets) struct Held(Vec<KeyCode>);

impl Held {
    fn follow(&mut self, key: &KeyboardInput) {
        let modifier = matches!(
            key.key_code,
            KeyCode::ControlLeft
                | KeyCode::ControlRight
                | KeyCode::SuperLeft
                | KeyCode::SuperRight
                | KeyCode::ShiftLeft
                | KeyCode::ShiftRight
                | KeyCode::AltLeft
                | KeyCode::AltRight
        );
        if modifier {
            self.0.retain(|held| *held != key.key_code);
            if key.state == KeyState::Pressed {
                self.0.push(key.key_code);
            }
        }
    }

    fn any(&self, keys: [KeyCode; 2]) -> bool {
        self.0.iter().any(|held| keys.contains(held))
    }

    /// Ctrl, or Cmd on a Mac: with C, X, V and A. Not AltGr, which Windows
    /// sends as Ctrl with right Alt, and which types letters such as @ and €.
    fn command(&self) -> bool {
        let alt_gr = self.any([KeyCode::AltRight, KeyCode::AltRight]);
        (self.any([KeyCode::ControlLeft, KeyCode::ControlRight]) && !alt_gr)
            || self.any([KeyCode::SuperLeft, KeyCode::SuperRight])
    }

    pub(super) fn shift(&self) -> bool {
        self.any([KeyCode::ShiftLeft, KeyCode::ShiftRight])
    }

    /// Ctrl, or Option on a Mac: moves and deletes by words.
    fn word(&self) -> bool {
        self.any([KeyCode::ControlLeft, KeyCode::ControlRight])
            || self.any([KeyCode::AltLeft, KeyCode::AltRight])
    }

    /// Cmd: with Left and Right, to the start or end, as on a Mac.
    fn line(&self) -> bool {
        self.any([KeyCode::SuperLeft, KeyCode::SuperRight])
    }
}

/// What the box says with its caret, as last left: at the end if its owner
/// changed the writing since.
pub(super) fn current(text_box: &TextBox, cursor: &Cursor) -> Edited {
    if cursor.text != text_box.text {
        return Edited::at_end(&text_box.text);
    }
    let len = text_box.text.chars().count();
    Edited {
        text: text_box.text.clone(),
        caret: cursor.caret.min(len),
        anchor: cursor.anchor.min(len),
    }
}

/// Keeps an edit; `true` if the writing changed. `now` is when, to keep the
/// caret solid a while if it moved.
pub(super) fn keep(edited: Edited, text_box: &mut TextBox, cursor: &mut Cursor, now: f32) -> bool {
    let moved = (cursor.caret, cursor.anchor) != (edited.caret, edited.anchor);
    let changed = edited.text != text_box.text;
    if moved || changed {
        cursor.moved_at = now;
    }
    (cursor.caret, cursor.anchor) = (edited.caret, edited.anchor);
    if changed {
        text_box.text.clone_from(&edited.text);
    }
    cursor.text = edited.text;
    changed
}

/// What one key does to the box; `false` if it is not one of ours.
fn press(key: &KeyboardInput, held: &Held, edited: &mut Edited, digits_only: bool) -> bool {
    let select = held.shift();
    let motion = |plain, word| if held.word() { word } else { plain };
    match &key.logical_key {
        Key::ArrowLeft if held.line() => edited.move_caret(Motion::Home, select),
        Key::ArrowRight if held.line() => edited.move_caret(Motion::End, select),
        Key::ArrowLeft => edited.move_caret(motion(Motion::Left, Motion::WordLeft), select),
        Key::ArrowRight => edited.move_caret(motion(Motion::Right, Motion::WordRight), select),
        Key::Home => edited.move_caret(Motion::Home, select),
        Key::End => edited.move_caret(Motion::End, select),
        Key::Backspace => edited.backspace(held.word()),
        Key::Delete => edited.delete(held.word()),
        _ if held.command() => match key.key_code {
            KeyCode::KeyA => edited.select_all(),
            KeyCode::KeyC => copy(&edited.selected()),
            KeyCode::KeyX => copy(&edited.cut()),
            KeyCode::KeyV => {
                if let Some(pasted) = clipboard::paste() {
                    edited.insert(&allowed(&pasted, digits_only));
                }
            }
            // Other shortcuts type nothing.
            _ => {}
        },
        Key::Space => edited.insert(&allowed(" ", digits_only)),
        Key::Character(typed) => edited.insert(&allowed(typed, digits_only)),
        _ => return false,
    }
    true
}

fn copy(text: &str) {
    if !text.is_empty() {
        clipboard::copy_to_clipboard(text);
    }
}

/// Who has the keyboard, and what they type.
#[allow(clippy::too_many_arguments)]
pub(in crate::widgets) fn take_text(
    mut focus: ResMut<TextFocus>,
    mut boxes: Query<(&mut TextBox, &mut Cursor)>,
    mut pressed: MessageReader<ButtonPressed>,
    noted: Res<PressedThisFrame>,
    mut typed: MessageReader<KeyboardInput>,
    mut keys: ResMut<ButtonInput<KeyCode>>,
    mut edited_boxes: MessageWriter<TextEdited>,
    mut soft_keys: MessageWriter<SoftKeyPressed>,
    mut told_soft_keyboard: Local<(Option<Entity>, Edited)>,
    mut held: ResMut<Held>,
    preferences: Res<Preferences>,
    time: Res<Time<Real>>,
) {
    let now = time.elapsed_secs();
    let on_box = pressed
        .read()
        .map(|press| press.0)
        .filter(|&entity| boxes.contains(entity))
        .last();
    let holds_keys = |focus: &TextFocus, boxes: &Query<(&mut TextBox, &mut Cursor)>| {
        focus
            .0
            .and_then(|entity| boxes.get(entity).ok())
            .is_some_and(|(text_box, _)| text_box.takes_keys)
    };
    if on_box.is_some() {
        focus.0 = on_box;
    } else if noted.0 && holds_keys(&focus, &boxes) {
        focus.0 = None;
    }
    let typed: Vec<KeyboardInput> = typed.read().cloned().collect();
    let soft: Vec<_> = soft_keyboard::take_keys();
    let gave_up = !soft.is_empty();
    for key in soft {
        soft_keys.write(SoftKeyPressed(key));
    }
    let pasted = clipboard::arrived();
    let Some(entity) = focus.0 else {
        typed.iter().for_each(|key| held.follow(key));
        return;
    };
    let Ok((mut text_box, mut cursor)) = boxes.get_mut(entity) else {
        focus.0 = None;
        return;
    };

    let digits_only = text_box.digits_only;
    let mut edited = current(&text_box, &cursor);
    let before = edited.clone();
    let controls = &preferences.controls;
    for key in &typed {
        held.follow(key);
        // The debug menu's key opens and closes it.
        if key.state == KeyState::Pressed && !controls.uses_key(Action::DebugMenu, key.key_code) {
            press(key, &held, &mut edited, digits_only);
        }
    }
    if let Some(pasted) = pasted {
        edited.insert(&allowed(&pasted, digits_only));
    }
    if edited == before
        && let Some((typed, caret)) = soft_keyboard::typed(&edited.text)
    {
        // The browser's hidden input, typed into on a device without keys.
        edited.text = allowed(&typed, digits_only);
        let len = edited.text.chars().count();
        edited.caret = caret.min(len);
        edited.anchor = edited.caret;
    }
    if told_soft_keyboard.0 != Some(entity) || told_soft_keyboard.1 != edited {
        soft_keyboard::show_text(&edited.text, edited.anchor, edited.caret);
        *told_soft_keyboard = (Some(entity), edited.clone());
    }
    if keep(edited, &mut text_box, &mut cursor, now) {
        edited_boxes.write(TextEdited(entity));
    }
    if text_box.takes_keys {
        let done = keys.just_pressed(KeyCode::Enter)
            || keys.just_pressed(KeyCode::NumpadEnter)
            || keys.just_pressed(KeyCode::Escape)
            || gave_up;
        if done {
            focus.0 = None;
        }
        keys.reset_all();
    }
}

#[cfg(test)]
mod tests;
