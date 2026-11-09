//! Remembering the server that welcomed the player, for Play and Connect, and
//! forgetting it when Logout goes back to the usual one.

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

/// When Play goes back to the usual server (Logout, or Connect forgetting the
/// chosen one), it stays there next time the game starts too, not only once
/// the usual server has welcomed the player.
pub(super) fn forget_when_unchosen(
    chosen: Res<ChosenServer>,
    mut preferences: ResMut<Preferences>,
) {
    if chosen.is_changed() && chosen.0.is_none() && preferences.last_server.is_some() {
        preferences.connected_to(None);
    }
}
