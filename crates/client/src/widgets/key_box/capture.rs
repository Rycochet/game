//! Waiting for a key: the first key that isn't a modifier, with the modifiers
//! held then (Shift + G), or a modifier pressed and let go with nothing else
//! (Shift). Followed from the keys' own events, as a waiting box takes every key
//! from the game each frame.

use bevy::input::ButtonState as KeyState;
use bevy::input::keyboard::{KeyboardFocusLost, KeyboardInput};
use bevy::prelude::*;

use super::super::text_box::PressedThisFrame;
use super::super::{ButtonPressed, ButtonState, TextFocus};
use super::{KeyBox, KeyChosen};
use crate::common::{Binding, Mods};

/// What waiting for a key has come to.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum Outcome {
    Cancelled,
    Chosen(Option<Binding>),
}

/// The box waiting for a key, if any, and what is held so far.
#[derive(Resource, Default, Debug)]
pub(in crate::widgets) struct KeyWaiting {
    pub(in crate::widgets) entity: Option<Entity>,
    pub(super) held: Mods,
    /// A modifier pressed with nothing after it yet, which letting go binds.
    lone: Option<KeyCode>,
}

impl KeyWaiting {
    pub(super) fn start(&mut self, entity: Entity, held: Mods) {
        *self = Self {
            entity: Some(entity),
            held,
            lone: None,
        };
    }

    /// One key going down (not repeated) or up.
    pub(super) fn step(&mut self, key: KeyCode, down: bool) -> Option<Outcome> {
        let modifier = Mods::of_key(key);
        if !down {
            let modifier = modifier?;
            self.held = self.held.without(modifier);
            let lone = self.lone.take()?;
            return (Mods::of_key(lone) == Some(modifier))
                .then(|| Outcome::Chosen(Some(Binding::new(lone, self.held))));
        }
        match (key, modifier) {
            (KeyCode::Escape, _) => Some(Outcome::Cancelled),
            (KeyCode::Backspace, _) => Some(Outcome::Chosen(None)),
            (_, Some(modifier)) => {
                self.held = self.held.with(modifier);
                self.lone = Some(key);
                None
            }
            _ if Binding::can_bind(key) => {
                Some(Outcome::Chosen(Some(Binding::new(key, self.held))))
            }
            _ => {
                self.lone = None;
                None
            }
        }
    }
}

/// A press on a box makes it wait (or stop, if it was); any other press stops
/// it. While one waits it has every key.
#[allow(clippy::too_many_arguments)]
pub(in crate::widgets) fn take_keys(
    mut waiting: ResMut<KeyWaiting>,
    mut boxes: Query<(&mut KeyBox, &ButtonState)>,
    mut pressed: MessageReader<ButtonPressed>,
    noted: Res<PressedThisFrame>,
    mut events: MessageReader<KeyboardInput>,
    mut focus_lost: MessageReader<KeyboardFocusLost>,
    mut keys: ResMut<ButtonInput<KeyCode>>,
    mut text_focus: ResMut<TextFocus>,
    mut chosen: MessageWriter<KeyChosen>,
) {
    let events: Vec<KeyboardInput> = events.read().cloned().collect();
    let on_box = pressed
        .read()
        .map(|press| press.0)
        .filter(|&entity| boxes.contains(entity))
        .last();
    match on_box {
        Some(entity) if waiting.entity == Some(entity) => waiting.entity = None,
        Some(entity) => {
            waiting.start(entity, Mods::held(&keys));
            text_focus.0 = None;
        }
        None if noted.0 => waiting.entity = None,
        None => {}
    }
    if focus_lost.read().count() > 0 {
        waiting.held = Mods::NONE;
    }
    let Some(entity) = waiting.entity else {
        return;
    };
    let usable = boxes
        .get(entity)
        .is_ok_and(|(_, state)| !state.is_disabled());
    if !usable || text_focus.0.is_some() {
        waiting.entity = None;
        return;
    }
    for event in events.iter().filter(|event| !event.repeat) {
        let down = event.state == KeyState::Pressed;
        match waiting.step(event.key_code, down) {
            Some(Outcome::Cancelled) => waiting.entity = None,
            Some(Outcome::Chosen(binding)) => {
                if let Ok((mut key_box, _)) = boxes.get_mut(entity) {
                    key_box.binding = binding;
                    key_box.warned = false;
                }
                chosen.write(KeyChosen(entity, binding));
                waiting.entity = None;
            }
            None => continue,
        }
        break;
    }
    // Esc included: it was this box's, not the menu's.
    keys.reset_all();
}
