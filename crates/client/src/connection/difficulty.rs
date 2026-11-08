//! Difficulty (`shared::difficulty`): keeping what the server says of it in
//! `common::PlayDifficulty`, and (native) asking the built-in server to allow
//! what Server Options' tickboxes say.

use bevy::prelude::*;
use shared::protocol::ServerMessage;

use super::FromServer;
use crate::common::PlayDifficulty;

/// The character's difficulty and the ones allowed, as the server says. (The
/// allowed ones for a new character come with `Characters`, in `receive`.)
pub(super) fn keep_difficulty(
    mut from_server: MessageReader<FromServer>,
    mut difficulty: ResMut<PlayDifficulty>,
) {
    for FromServer(message) in from_server.read() {
        if let ServerMessage::Difficulty { current, allowed } = message {
            // Always marked changed, so a refused choice is put back.
            *difficulty = PlayDifficulty {
                current: *current,
                allowed: *allowed,
            };
        }
    }
}

/// Native: while playing on the built-in server, asks it to allow Server
/// Options' difficulties whenever they differ from what it was last asked,
/// and again after each welcome. `HOW_DIFFICULTIES`, if set, wins there.
#[cfg(not(target_arch = "wasm32"))]
pub(super) fn ask_difficulties(
    connection: Option<Res<super::ServerConnection>>,
    status: Res<crate::common::ServerStatus>,
    preferences: Res<crate::common::Preferences>,
    mut from_server: MessageReader<FromServer>,
    mut to_server: MessageWriter<super::ToServer>,
    mut asked: Local<Option<shared::difficulty::Difficulties>>,
) {
    let welcomed = from_server
        .read()
        .any(|FromServer(message)| matches!(message, ServerMessage::Welcome { .. }));
    let on_its_own = connection.is_some_and(|connection| connection.local);
    if welcomed || !on_its_own || !status.is_connected() {
        *asked = None;
        if !welcomed {
            return;
        }
    }
    let wanted = preferences.server_difficulties;
    if *asked != Some(wanted) {
        let message = shared::protocol::ClientMessage::SetDifficulties(wanted);
        to_server.write(super::ToServer(message));
        *asked = Some(wanted);
    }
}
