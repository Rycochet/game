//! The requester asking, before Ironman becomes the player's own difficulty,
//! whether they are happy for death to be permanent.

use bevy::prelude::*;

use super::super::{ABOVE_MENU_LAYER, Action, BUTTON_ROW, FONT, Layer, MENU_WIDTH, SystemMenu};
use crate::common::IRONMAN_QUESTION;
use crate::widgets::{ButtonState, ButtonStyle, INK, Pressable, spawn_button, spawn_dialog};

/// Above Settings.
const POPUP_LAYER: i32 = ABOVE_MENU_LAYER + 1;
/// How wide its words run before wrapping.
const WORDS_WIDTH: f32 = 420.0;

pub(in crate::system_menu) fn spawn_ironman_requester(commands: &mut Commands) {
    let dialog = spawn_dialog(commands, POPUP_LAYER, "Ironman");
    commands
        .entity(dialog.layer)
        .insert((Layer(&[SystemMenu::Ironman]), Action::Back));
    commands.entity(dialog.content).with_child((
        Text::new(IRONMAN_QUESTION),
        FONT.font(),
        TextColor(INK),
        Node {
            max_width: px(WORDS_WIDTH),
            ..default()
        },
    ));
    commands.entity(dialog.buttons).with_children(|row| {
        let node = Node {
            width: px(MENU_WIDTH / 2.0),
            height: px(BUTTON_ROW),
            ..default()
        };
        for (label, action, style) in [
            ("Yes, Ironman", Action::ChooseIronman, ButtonStyle::Main),
            ("No", Action::Back, ButtonStyle::Plain),
        ] {
            let id = match action {
                Action::Back => crate::widgets::UiId::new("ironman", "no"),
                _ => action.ui_id("ironman"),
            };
            spawn_button(row, ButtonState::Normal, label, FONT, node.clone())
                .insert((action, style, Pressable, id));
        }
    });
}
