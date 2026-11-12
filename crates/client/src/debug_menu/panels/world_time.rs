//! The World Time panel: buttons moving the world's time by steps either way,
//! and a box with the time in it that moves it to exactly what is typed; below
//! them, how fast it runs (`world_rate`).

use crate::widgets::UiId;
use bevy::input::ButtonState as KeyState;
use bevy::input::keyboard::KeyboardInput;
use bevy::prelude::*;
use shared::protocol::ClientMessage;

use super::time_text::{read_time, time_text};
use super::world_rate::rate_named;
use crate::common::ServerClock;
use crate::connection::ToServer;
use crate::dom::Template;
use crate::html;
use crate::widgets::{ButtonPressed, SoftKey, SoftKeyPressed, TextBox, TextFocus};

const HOUR: f64 = 60.0;
const DAY: f64 = 24.0 * HOUR;
/// The buttons either side of the box, by their names in the markup, and
/// the minutes each moves the clock.
const STEPS: [(&str, f64); 12] = [
    ("back_1d", -DAY),
    ("back_12h", -12.0 * HOUR),
    ("back_1h", -HOUR),
    ("back_30m", -30.0),
    ("back_10m", -10.0),
    ("back_1m", -1.0),
    ("on_1m", 1.0),
    ("on_10m", 10.0),
    ("on_30m", 30.0),
    ("on_1h", HOUR),
    ("on_12h", 12.0 * HOUR),
    ("on_1d", DAY),
];

/// The panel's contents, as markup: buttons and box touching (owner's
/// choice), and below them how fast the world's time runs.
pub(super) static WORLD_TIME: Template = html!(
    "debug_menu.world_time",
    r#"
<form id="debug.world_time">
  <panel id="panel" moveable>
    <div class="tight">
    <span class="joined" nowrap valign="middle">
      <small class="joined" nowrap>
        <button id="back_1d">-1d</button>
        <button id="back_12h">-12h</button>
        <button id="back_1h">-1h</button>
        <button id="back_30m">-30m</button>
        <button id="back_10m">-10m</button>
        <button id="back_1m">-1m</button>
      </small>
      <input id="time" type="text" width="150"/>
      <small class="joined" nowrap>
        <button id="on_1m">+1m</button>
        <button id="on_10m">+10m</button>
        <button id="on_30m">+30m</button>
        <button id="on_1h">+1h</button>
        <button id="on_12h">+12h</button>
        <button id="on_1d">+1d</button>
      </small>
    </span>
    <small class="joined" nowrap>
      <button id="rate_quarter">x1/4</button>
      <button id="rate_half">x1/2</button>
      <button id="rate_1">x1</button>
      <button id="rate_2">x2</button>
      <button id="rate_10">x10</button>
      <button id="rate_60">x60</button>
      <button id="rate_600">x600</button>
    </small>
    </div>
  </panel>
</form>
"#
);

/// What a built part of the panel is known by, from its id.
pub(super) fn bind(bound: &mut EntityCommands, id: &UiId) {
    let name = id.parts().1;
    if name == "time" {
        bound.insert(TimeBox);
    } else if let Some((_, minutes)) = STEPS.iter().find(|(known, _)| *known == name) {
        bound.insert(TimeStep(*minutes));
    } else if let Some(rate) = rate_named(name) {
        bound.insert(rate);
    }
}

/// A button moving the world's clock by this many minutes.
#[derive(Component, Clone, Copy)]
pub(in crate::debug_menu) struct TimeStep(f64);

/// The box with the world's time in it.
#[derive(Component)]
pub(in crate::debug_menu) struct TimeBox;

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
        to_server.write(ToServer(ClientMessage::DevSetWorldClock {
            minutes: Some(clock.minutes + step.0),
            rate: None,
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
                to_server.write(ToServer(ClientMessage::DevSetWorldClock {
                    minutes: Some(minutes),
                    rate: None,
                }));
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
