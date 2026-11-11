//! Server Options' layer: a panel that a press outside closes, and its
//! title row with a close button. (The menu's other layers are markup.)

use bevy::prelude::*;

use super::super::{Action, CLOSE_MARGIN, Layer, SETTINGS_TITLE_ROW, SystemMenu};
use crate::widgets::{
    CLOSE_PIXELS, Pressable, UiId, panel_title, spawn_close_button, spawn_sized_layer,
};

/// A layer that presses outside its panel close.
pub(super) fn spawn_layer(
    commands: &mut Commands,
    depth: i32,
    shown: &'static [SystemMenu],
    width: Val,
    height: Val,
    content: Node,
) -> Entity {
    let (layer, panel) = spawn_sized_layer(commands, depth, width, height, content);
    commands.entity(layer).insert((Layer(shown), Action::Back));
    panel
}

pub(super) fn spawn_title_row(panel: &mut ChildSpawnerCommands, title: &str, form: &'static str) {
    panel
        .spawn(Node {
            height: px(SETTINGS_TITLE_ROW),
            flex_shrink: 0.0,
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            ..default()
        })
        .with_children(|row| {
            row.spawn(panel_title(title));
            row.spawn((
                Node {
                    position_type: PositionType::Absolute,
                    right: px(-CLOSE_MARGIN),
                    top: px((SETTINGS_TITLE_ROW - CLOSE_PIXELS) / 2.0 - CLOSE_MARGIN),
                    padding: UiRect::all(px(CLOSE_MARGIN)),
                    ..default()
                },
                Action::Back,
                Pressable,
                UiId::new(form, "close"),
            ))
            .with_children(|holder| {
                spawn_close_button(holder, default());
            });
        });
}
