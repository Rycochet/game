//! The Teleport to Gate panel: a row of buttons, one for each World Gate a
//! planet has, numbered from 1; pressing one asks the server to put the player
//! at that gate of the world they are in.

use bevy::prelude::*;
use shared::protocol::ClientMessage;
use shared::world_gate::WORLD_GATES;

use super::CHECK_ROW;
use crate::common::ServerDevTools;
use crate::connection::ToServer;
use crate::widgets::{ButtonPressed, ButtonState, Pressable, TextSize, UiId, spawn_button};

const GATE_PADDING: UiRect = UiRect {
    left: Val::Px(6.0),
    right: Val::Px(6.0),
    top: Val::Px(2.0),
    bottom: Val::Px(2.0),
};

/// A button for the World Gate of this number (from 0; it says one more).
#[derive(Component, Clone, Copy)]
pub(in crate::debug_menu) struct GateButton(u8);

pub(super) fn spawn_teleport_to_gate(content: &mut ChildSpawnerCommands) {
    content
        .spawn(Node {
            align_items: AlignItems::Center,
            column_gap: px(2),
            ..default()
        })
        .with_children(|row| {
            for gate in 0..WORLD_GATES as u8 {
                let node = Node {
                    height: px(CHECK_ROW),
                    padding: GATE_PADDING,
                    ..default()
                };
                let label = (gate + 1).to_string();
                spawn_button(row, ButtonState::Normal, label, TextSize::Small, node).insert((
                    GateButton(gate),
                    Pressable,
                    UiId::new("debug.teleport_to_gate", "gate").at(usize::from(gate)),
                ));
            }
        });
}

/// Asks the server to put the player at the pressed button's World Gate, in
/// the world they are in. A planet that couldn't place one leaves them be.
pub(in crate::debug_menu) fn press_gate_buttons(
    mut pressed: MessageReader<ButtonPressed>,
    buttons: Query<&GateButton>,
    mut to_server: MessageWriter<ToServer>,
) {
    for press in pressed.read() {
        if let Ok(button) = buttons.get(press.0) {
            to_server.write(ToServer(ClientMessage::DevGoToWorld {
                seed: None,
                gate: Some(button.0),
            }));
        }
    }
}

/// Greys the buttons while the server doesn't take dev tools.
pub(in crate::debug_menu) fn update_gate_buttons(
    dev_tools: Res<ServerDevTools>,
    mut buttons: Query<&mut ButtonState, With<GateButton>>,
) {
    let state = if dev_tools.0 {
        ButtonState::Normal
    } else {
        ButtonState::Disabled
    };
    for mut button in &mut buttons {
        button.set_if_neq(state);
    }
}
