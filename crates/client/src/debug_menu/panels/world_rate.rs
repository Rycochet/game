//! The World Time panel's second row: buttons for how fast the world's time
//! runs against the server's clock, the one it runs at chosen.

use crate::widgets::UiId;
use bevy::prelude::*;
use shared::protocol::ClientMessage;
use shared::world::ClockRate;

use super::CHECK_ROW;
use crate::common::ServerClock;
use crate::connection::ToServer;
use crate::widgets::{ButtonPressed, ButtonState, Pressable, TextSize, spawn_button};

/// The rates offered, by name, as game minutes for real ones.
const RATES: [(&str, u32, u32); 7] = [
    ("x1/4", 1, 4),
    ("x1/2", 1, 2),
    ("x1", 1, 1),
    ("x2", 2, 1),
    ("x10", 10, 1),
    ("x60", 60, 1),
    ("x600", 600, 1),
];
const RATE_PADDING: UiRect = UiRect {
    left: Val::Px(6.0),
    right: Val::Px(6.0),
    top: Val::Px(2.0),
    bottom: Val::Px(2.0),
};

/// A button setting the world's time to run at this rate.
#[derive(Component, Clone, Copy)]
pub(in crate::debug_menu) struct RateButton(ClockRate);

pub(super) fn spawn_world_rate(content: &mut ChildSpawnerCommands) {
    content
        .spawn(Node {
            align_items: AlignItems::Center,
            margin: UiRect::top(Val::Px(4.0)),
            ..default()
        })
        .with_children(|row| {
            for (at, (label, game, real)) in RATES.into_iter().enumerate() {
                let rate = ClockRate::new(game, real).expect("none of them stop time");
                let node = Node {
                    height: px(CHECK_ROW),
                    padding: RATE_PADDING,
                    ..default()
                };
                spawn_button(row, ButtonState::Normal, label, TextSize::Small, node).insert((
                    RateButton(rate),
                    Pressable,
                    UiId::new("debug.world_time", "rate").at(at),
                ));
            }
        });
}

/// Asks the server to run the world's time at the pressed button's rate.
pub(in crate::debug_menu) fn press_rate_buttons(
    mut pressed: MessageReader<ButtonPressed>,
    buttons: Query<&RateButton>,
    mut to_server: MessageWriter<ToServer>,
) {
    for press in pressed.read() {
        if let Ok(button) = buttons.get(press.0) {
            to_server.write(ToServer(ClientMessage::DevSetWorldClock {
                minutes: None,
                rate: Some(button.0),
            }));
        }
    }
}

/// Shows the rate the world's time runs at as chosen.
pub(in crate::debug_menu) fn update_rate_buttons(
    clock: Res<ServerClock>,
    mut buttons: Query<(&RateButton, &mut ButtonState)>,
) {
    for (button, mut state) in &mut buttons {
        state.set_if_neq(if button.0 == clock.time.rate {
            ButtonState::Selected
        } else {
            ButtonState::Normal
        });
    }
}
