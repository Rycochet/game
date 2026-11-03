//! Sending steps and messages, counting map changes, and hanging up on quitting.

use bevy::prelude::*;
use shared::protocol::ServerMessage;

use super::prediction::Prediction;
use super::{FromServer, ServerConnection, ToServer};
use crate::player::Stepped;

/// Sends the steps the player took this frame, keeping them until the
/// server confirms them.
pub(super) fn send_steps(
    mut stepped: MessageReader<Stepped>,
    mut prediction: ResMut<Prediction>,
    mut to_server: MessageWriter<ToServer>,
) {
    for &Stepped { step, moved } in stepped.read() {
        to_server.write(ToServer(prediction.step(step, moved)));
    }
}

/// The server has put the player on another map (applied by `site` this
/// frame): steps from now on are on it.
pub(super) fn count_map_changes(
    mut from_server: MessageReader<FromServer>,
    mut prediction: ResMut<Prediction>,
) {
    for FromServer(message) in from_server.read() {
        if matches!(
            message,
            ServerMessage::EnterLevel(_)
                | ServerMessage::OnWorldMap { .. }
                | ServerMessage::Teleported { .. }
        ) {
            prediction.changed_map();
        }
    }
}

/// Sends what other systems wrote this frame.
pub(super) fn send(connection: Res<ServerConnection>, mut to_server: MessageReader<ToServer>) {
    for message in to_server.read() {
        connection.link.send(&message.0);
    }
}

/// Quitting: hangs up, so the server knows the player has gone (and an
/// embedded one saves and stops; this waits for it).
pub(super) fn hang_up_on_exit(mut commands: Commands, mut exit: MessageReader<AppExit>) {
    if exit.read().next().is_some() {
        commands.remove_resource::<ServerConnection>();
    }
}
