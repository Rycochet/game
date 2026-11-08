//! Sneak, a toggle: pressed on the bar it is asked for at once (no target, no
//! waiting its turn), and the server says whether it is on.

use bevy::prelude::*;
use shared::protocol::ServerMessage;

use crate::common::Sneaking;
use crate::connection::FromServer;

/// Keeps `common::Sneaking` as the server says: off on joining, as it isn't
/// kept.
pub(super) fn take_sneaking(
    mut from_server: MessageReader<FromServer>,
    mut sneaking: ResMut<Sneaking>,
) {
    for FromServer(message) in from_server.read() {
        match message {
            ServerMessage::Sneaking(on) => sneaking.set_if_neq(Sneaking(*on)),
            ServerMessage::Welcome { .. } => sneaking.set_if_neq(Sneaking(false)),
            _ => false,
        };
    }
}
