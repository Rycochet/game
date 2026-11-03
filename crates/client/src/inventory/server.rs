//! Taking what the server says is carried, and how big it lets stacks get.

use bevy::prelude::*;
use shared::item::StackLimits;
use shared::protocol::ServerMessage;

use super::Carried;
use crate::connection::FromServer;

/// How big the server lets stacks get, from `Welcome`.
#[derive(Resource, Default)]
pub(super) struct Stacks(pub StackLimits);

pub(super) fn read_server(
    mut from_server: MessageReader<FromServer>,
    mut carried: ResMut<Carried>,
    mut stacks: ResMut<Stacks>,
) {
    for message in from_server.read() {
        match &message.0 {
            ServerMessage::Inventory(inventory) => carried.0 = inventory.clone(),
            ServerMessage::Welcome { stacks: limits, .. } => stacks.0 = *limits,
            _ => {}
        }
    }
}
