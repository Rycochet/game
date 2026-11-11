//! The World Time panel's second row: buttons for how fast the world's time
//! runs against the server's clock, the one it runs at chosen.

use bevy::prelude::*;
use shared::protocol::ClientMessage;
use shared::world::ClockRate;

use crate::common::ServerClock;
use crate::connection::ToServer;
use crate::widgets::{ButtonPressed, ButtonState};

/// The rates offered, by their buttons' names in the markup, as game minutes
/// for real ones.
const RATES: [(&str, u32, u32); 7] = [
    ("rate_quarter", 1, 4),
    ("rate_half", 1, 2),
    ("rate_1", 1, 1),
    ("rate_2", 2, 1),
    ("rate_10", 10, 1),
    ("rate_60", 60, 1),
    ("rate_600", 600, 1),
];

/// A button setting the world's time to run at this rate.
#[derive(Component, Clone, Copy)]
pub(in crate::debug_menu) struct RateButton(ClockRate);

/// The button with this name in the markup, if it is one of the rates'.
pub(super) fn rate_named(name: &str) -> Option<RateButton> {
    let &(_, game, real) = RATES.iter().find(|(known, ..)| *known == name)?;
    ClockRate::new(game, real).map(RateButton)
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
