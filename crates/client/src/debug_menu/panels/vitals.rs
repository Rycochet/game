//! The Vitals panel: Hurt, Spend Mana, Heal Fully and Bonus, each asking the
//! server to change the character's Health and Mana (`DevVitals`); the vitals
//! circle shows the answer.

use bevy::prelude::*;
use shared::protocol::{ClientMessage, DevVitals};

use super::{FONT, row};
use crate::common::ServerDevTools;
use crate::connection::ToServer;
use crate::widgets::{BUTTON_PADDING, ButtonPressed, ButtonState, Pressable, UiId, spawn_button};

/// How much Hurt takes and Spend Mana spends.
const STEP: u32 = 5;

/// A button sending this to the server.
#[derive(Component, Clone, Copy)]
pub(in crate::debug_menu) struct VitalsButton(DevVitals);

pub(super) fn spawn_vitals_panel(content: &mut ChildSpawnerCommands) {
    content.spawn(row()).with_children(|row| {
        for (label, vitals, name) in [
            ("Hurt 5", DevVitals::Hurt(STEP), "hurt"),
            ("Spend 5 Mana", DevVitals::SpendMana(STEP), "spend_mana"),
            ("Heal Fully", DevVitals::Fill, "heal_fully"),
            ("Bonus", DevVitals::Bonus, "bonus"),
        ] {
            let node = Node {
                padding: BUTTON_PADDING,
                ..default()
            };
            spawn_button(row, ButtonState::Normal, label, FONT, node).insert((
                VitalsButton(vitals),
                Pressable,
                UiId::new("debug.vitals", name),
            ));
        }
    });
}

/// Sends what the pressed button says.
pub(in crate::debug_menu) fn press_vitals_buttons(
    mut pressed: MessageReader<ButtonPressed>,
    buttons: Query<&VitalsButton>,
    mut to_server: MessageWriter<ToServer>,
) {
    for press in pressed.read() {
        if let Ok(button) = buttons.get(press.0) {
            to_server.write(ToServer(ClientMessage::DevVitals(button.0)));
        }
    }
}

/// Greys the buttons while the server doesn't take dev tools.
pub(in crate::debug_menu) fn update_vitals_buttons(
    dev_tools: Res<ServerDevTools>,
    mut buttons: Query<&mut ButtonState, With<VitalsButton>>,
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
