//! Being killed: a requester saying so, which blocks the game until it is
//! closed (owner, 2026-11-07, for now). The server has already put the player
//! on the world map, whole again; on Ironman it has deleted the character, and
//! the creation screen follows.

use bevy::prelude::*;
use shared::protocol::ServerMessage;

use crate::connection::FromServer;
use crate::widgets::{
    ButtonPressed, ButtonState, ButtonStyle, INK, Pressable, TextSize, UiId, layer_z, spawn_button,
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

/// Its words.
#[derive(Component)]
pub(super) struct KilledWords;

const TRY_AGAIN: &str = "You have been killed, try again!";
const FOR_GOOD: &str = "You have been killed. On Ironman death is permanent: your character \
     and everything they had are gone. Make a new one to play again.";

pub(super) fn spawn_killed(mut commands: Commands) {
    let dialog = spawn_dialog(&mut commands, KILLED_LAYER, "Killed");
    commands.entity(dialog.layer).insert(KilledRequester);
    commands.entity(dialog.content).with_child((
        KilledWords,
        Text::new(TRY_AGAIN),
        FONT.font(),
        TextColor(INK),
        Node {
            max_width: px(420.0),
            ..default()
        },
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
    mut requester: Single<(&mut Visibility, &mut GlobalZIndex), With<KilledRequester>>,
    mut words: Single<&mut Text, With<KilledWords>>,
) {
    let (visibility, z) = &mut *requester;
    for FromServer(message) in from_server.read() {
        if let ServerMessage::Killed { permanent } = message {
            words.0 = if *permanent { FOR_GOOD } else { TRY_AGAIN }.into();
            // Over the creation screen that follows a permanent death.
            let depth = if *permanent {
                KILLED_LAYER + 1
            } else {
                KILLED_LAYER
            };
            z.set_if_neq(GlobalZIndex(layer_z(depth)));
            visibility.set_if_neq(Visibility::Inherited);
        }
    }
    if pressed.read().any(|press| oks.contains(press.0)) {
        visibility.set_if_neq(Visibility::Hidden);
    }
}
