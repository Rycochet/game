//! The character's name, shown in the middle of the tree, as the server says it
//! on joining (`ServerMessage::Welcome`).

use bevy::prelude::*;
use shared::protocol::ServerMessage;

use crate::connection::FromServer;

#[derive(Resource, Clone, Debug, Default, PartialEq, Eq)]
pub(super) struct OwnName(pub String);

pub(super) fn read_name(mut from_server: MessageReader<FromServer>, mut name: ResMut<OwnName>) {
    for message in from_server.read() {
        if let ServerMessage::Welcome { name: told, .. } = &message.0 {
            name.set_if_neq(OwnName(told.clone()));
        }
    }
}
