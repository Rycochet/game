//! Taking what the server says is carried.

use bevy::prelude::*;
use shared::protocol::ServerMessage;

use super::Carried;
use crate::connection::FromServer;

pub(super) fn read_server(
    mut from_server: MessageReader<FromServer>,
    mut carried: ResMut<Carried>,
) {
    for message in from_server.read() {
        if let ServerMessage::Inventory(inventory) = &message.0 {
            carried.0 = inventory.clone();
        }
    }
}
