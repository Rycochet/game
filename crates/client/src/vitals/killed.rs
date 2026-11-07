//! Being killed: a requester saying so, which blocks the game until it is
//! closed (owner, 2026-11-07, for now). The server has already put the player
//! on the world map, whole again.

use bevy::prelude::*;
use shared::protocol::ServerMessage;

use crate::connection::FromServer;
use crate::widgets::{
    ButtonPressed, ButtonState, ButtonStyle, INK, Pressable, TextSize, UiId, spawn_button,
    spawn_dialog,
};

/// With the system menu's own layer: it blocks the game, not the menu.
const KILLED_LAYER: i32 = 0;
const FONT: TextSize = TextSize::Medium;

/// The requester's layer.
#[derive(Component)]
pub(super) struct KilledRequester;

/// Its OK button.
#[derive(Component)]
pub(super) struct KilledOk;

pub(super) fn spawn_killed(mut commands: Commands) {
    let dialog = spawn_dialog(&mut commands, KILLED_LAYER, "Killed");
    commands.entity(dialog.layer).insert(KilledRequester);
    commands.entity(dialog.content).with_child((
        Text::new("You have been killed, try again!"),
        FONT.font(),
        TextColor(INK),
    ));
    commands.entity(dialog.buttons).with_children(|row| {
        let node = Node {
            width: px(120.0),
            height: px(36.0),
            ..default()
        };
        spawn_button(row, ButtonState::Normal, "OK", FONT, node).insert((
            KilledOk,
            UiId::new("killed", "ok"),
            ButtonStyle::Main,
            Pressable,
        ));
    });
}

/// Shows it when the server says the player was killed, and closes it on OK.
pub(super) fn show_killed(
    mut from_server: MessageReader<FromServer>,
    mut pressed: MessageReader<ButtonPressed>,
    oks: Query<(), With<KilledOk>>,
    mut requester: Single<&mut Visibility, With<KilledRequester>>,
) {
    for FromServer(message) in from_server.read() {
        if matches!(message, ServerMessage::Killed) {
            requester.set_if_neq(Visibility::Inherited);
        }
    }
    if pressed.read().any(|press| oks.contains(press.0)) {
        requester.set_if_neq(Visibility::Hidden);
    }
}
