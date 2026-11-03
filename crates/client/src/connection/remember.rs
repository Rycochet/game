//! Remembering the server that welcomed the player, for Play and Connect.

use bevy::prelude::*;
use shared::protocol::ServerMessage;

use super::FromServer;
use crate::common::{ChosenServer, Preferences};

/// On a welcome: the chosen server goes to the top of Connect's list, and Play
/// connects there next time (or to the usual one, if none was chosen).
pub(super) fn remember_server(
    mut from_server: MessageReader<FromServer>,
    chosen: Res<ChosenServer>,
    mut preferences: ResMut<Preferences>,
) {
    let welcomed = from_server
        .read()
        .any(|FromServer(message)| matches!(message, ServerMessage::Welcome { .. }));
    if welcomed {
        let mut changed = preferences.clone();
        changed.connected_to(chosen.0.as_ref());
        preferences.set_if_neq(changed);
    }
}
