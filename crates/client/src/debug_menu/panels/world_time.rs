//! The World Time panel: buttons moving the world's clock by steps either way,
//! and a box with the time in it that moves the clock to exactly what is typed.

use bevy::input::ButtonState as KeyState;
use bevy::input::keyboard::KeyboardInput;
use bevy::prelude::*;
use shared::protocol::ClientMessage;

use super::time_text::{read_time, time_text};
use super::{CHECK_ROW, FONT};
use crate::common::ServerClock;
use crate::connection::ToServer;
use crate::widgets::{
    ButtonPressed, ButtonState, Pressable, SoftKey, SoftKeyPressed, TextBox, TextFocus, TextSize,
    spawn_button, spawn_text_box,
};

const HOUR: f64 = 60.0;
const DAY: f64 = 24.0 * HOUR;
/// The buttons before the box, by name and minutes, then those after it.
const BACK: [(&str, f64); 6] = [
    ("-1d", -DAY),
    ("-12h", -12.0 * HOUR),
    ("-1h", -HOUR),
    ("-30m", -30.0),
    ("-10m", -10.0),
    ("-1m", -1.0),
];
const ON: [(&str, f64); 6] = [
    ("+1m", 1.0),
    ("+10m", 10.0),
    ("+30m", 30.0),
    ("+1h", HOUR),
    ("+12h", 12.0 * HOUR),
    ("+1d", DAY),
];
const STEP_FONT: TextSize = TextSize::Small;
const STEP_PADDING: UiRect = UiRect {
    left: Val::Px(6.0),
    right: Val::Px(6.0),
    top: Val::Px(2.0),
    bottom: Val::Px(2.0),
};
const BOX_WIDTH: f32 = 150.0;

/// A button moving the world's clock by this many minutes.
#[derive(Component, Clone, Copy)]
pub(in crate::debug_menu) struct TimeStep(f64);

/// The box with the world's time in it.
#[derive(Component)]
pub(in crate::debug_menu) struct TimeBox;

pub(super) fn spawn_world_time(content: &mut ChildSpawnerCommands) {
    content
        .spawn(Node {
            column_gap: px(2),
            align_items: AlignItems::Center,
            ..default()
        })
        .with_children(|row| {
            let step = |row: &mut ChildSpawnerCommands, (label, minutes): (&str, f64)| {
                let node = Node {
                    padding: STEP_PADDING,
                    ..default()
                };
                spawn_button(row, ButtonState::Normal, label, STEP_FONT, node)
                    .insert((TimeStep(minutes), Pressable));
            };
            for back in BACK {
                step(row, back);
            }
            let text_box = TextBox {
                takes_keys: true,
                ..default()
            };
            let node = Node {
                width: px(BOX_WIDTH),
                height: px(CHECK_ROW),
                margin: UiRect::horizontal(px(4)),
                ..default()
            };
            spawn_text_box(row, text_box, FONT, node).insert(TimeBox);
            for on in ON {
                step(row, on);
            }
        });
}

/// Asks the server to move the clock by the pressed button's step. It stops at
/// the world's beginning.
pub(in crate::debug_menu) fn press_time_steps(
    mut pressed: MessageReader<ButtonPressed>,
    steps: Query<&TimeStep>,
    mut boxes: Query<&mut TextBox, With<TimeBox>>,
    clock: Res<ServerClock>,
    mut to_server: MessageWriter<ToServer>,
) {
    for press in pressed.read() {
        let Ok(step) = steps.get(press.0) else {
            continue;
        };
        to_server.write(ToServer(ClientMessage::DevSetTime {
            minutes: clock.minutes + step.0,
        }));
        // Back to following the clock, if it was showing something unreadable.
        for mut text_box in &mut boxes {
            text_box.error = false;
        }
    }
}

/// Keeps the box on the world's time while it isn't being edited. When it gives
/// up the keyboard, the clock moves to what it says, unless that was Esc, which
/// puts the real time back; what can't be read stays, marked, and sends nothing.
#[allow(clippy::too_many_arguments)]
pub(in crate::debug_menu) fn edit_time_box(
    focus: Res<TextFocus>,
    mut boxes: Query<(Entity, &mut TextBox), With<TimeBox>>,
    mut keys: MessageReader<KeyboardInput>,
    mut soft_keys: MessageReader<SoftKeyPressed>,
    clock: Res<ServerClock>,
    mut had_focus: Local<Option<Entity>>,
    mut to_server: MessageWriter<ToServer>,
) {
    // Read from the keys themselves, as the box clears the pressed keys.
    let pressed_escape = keys
        .read()
        .any(|key| key.state == KeyState::Pressed && key.key_code == KeyCode::Escape);
    let escaped = soft_keys.read().fold(pressed_escape, |escaped, key| {
        escaped || key.0 == SoftKey::Escape
    });
    for (entity, mut text_box) in &mut boxes {
        let editing = focus.0 == Some(entity);
        if editing {
            text_box.error = false;
        } else if *had_focus == Some(entity) {
            if escaped {
                text_box.error = false;
            } else if let Some(minutes) = read_time(&text_box.text, clock.minutes) {
                to_server.write(ToServer(ClientMessage::DevSetTime { minutes }));
            } else {
                text_box.error = true;
            }
        }
        let now = time_text(clock.minutes);
        if !editing && !text_box.error && text_box.text != now {
            text_box.text = now;
        }
    }
    *had_focus = focus.0;
}
