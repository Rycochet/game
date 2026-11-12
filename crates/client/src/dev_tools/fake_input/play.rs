//! Fake input queued and played a step a frame.

use bevy::input::ButtonState;
use bevy::input::keyboard::{Key, KeyboardInput};
use bevy::input::mouse::{MouseButtonInput, MouseScrollUnit, MouseWheel};
use bevy::input::touch::{TouchInput, TouchPhase};
use bevy::prelude::*;
use bevy::window::PrimaryWindow;
use server::debug_api::{BrpResult, json};
use std::collections::VecDeque;

use super::FINGER;
use crate::dev_tools::FakeCursor;

/// One step of fake input, played in one frame.
#[derive(Clone, Debug, PartialEq)]
pub(super) enum Step {
    /// The pointer goes here (logical pixels), held by `FakeCursor`.
    MoveTo(Vec2),
    /// The pointer goes back to the real mouse.
    LetGo,
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
pub(super) struct FakeInput {
    pub(super) steps: VecDeque<Step>,
    pub(super) frame: u64,
    /// The step played this frame, if any.
    pub(super) played: Option<Step>,
}

/// Plays the next step, before Bevy reads this frame's input.
#[allow(clippy::too_many_arguments)]
pub(super) fn play(
    mut fake: ResMut<FakeInput>,
    window: Single<Entity, With<PrimaryWindow>>,
    mut cursor: ResMut<FakeCursor>,
    time: Res<Time<Real>>,
    mut mouse: MessageWriter<MouseButtonInput>,
    mut wheel: MessageWriter<MouseWheel>,
    mut touch: MessageWriter<TouchInput>,
    mut keys: MessageWriter<KeyboardInput>,
) {
    fake.frame += 1;
    fake.played = fake.steps.pop_front();
    let Some(step) = fake.played.clone() else {
        return;
    };
    let entity = *window;
    let now = time.elapsed_secs();
    if cursor.used_at.is_some() {
        cursor.used_at = Some(now);
    }
    match step {
        Step::MoveTo(at) => cursor.used(at, now),
        Step::LetGo => cursor.let_go(),
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
pub(super) fn queued(fake: &mut FakeInput, steps: impl IntoIterator<Item = Step>) -> BrpResult {
    fake.steps.extend(steps);
    Ok(json!({ "pending": fake.steps.len(), "wait_for_input": true }))
}
