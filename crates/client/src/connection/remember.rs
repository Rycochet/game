//! Remembering a server chosen in Connect once it has welcomed the player, at
//! the top of Connect's list.

use bevy::prelude::*;
use shared::protocol::ServerMessage;

use super::FromServer;
use crate::common::{ChosenServer, Preferences};

/// On a welcome from a server chosen in Connect: it goes to the top of
/// Connect's list.
pub(super) fn remember_server(
    mut from_server: MessageReader<FromServer>,
    chosen: Res<ChosenServer>,
    mut preferences: ResMut<Preferences>,
) {
    let welcomed = from_server
        .read()
        .any(|FromServer(message)| matches!(message, ServerMessage::Welcome { .. }));
    if let (true, Some(server)) = (welcomed, &chosen.0) {
        let mut changed = preferences.clone();
        changed.joined(server);
        preferences.set_if_neq(changed);
    }
}
