//! What the server says of bags.

use bevy::prelude::*;
use shared::protocol::ServerMessage;

use super::{BagAssets, Bags, set_bag};
use crate::connection::FromServer;

/// What the server says of bags. Anything out of sight isn't mentioned, so it stays as
/// remembered. A new map forgets them all: the server says again what is remembered.
pub(super) fn read_server(
    mut from_server: MessageReader<FromServer>,
    assets: Res<BagAssets>,
    mut bags: ResMut<Bags>,
    mut commands: Commands,
) {
    for FromServer(message) in from_server.read() {
        match message {
            ServerMessage::GroundSeen { x, y, load } => {
                set_bag(&mut bags, &mut commands, &assets, (*x, *y), *load);
            }
            ServerMessage::GroundGone { x, y } => {
                if let Some(old) = bags.0.remove(&(*x, *y)) {
                    commands.entity(old.root).despawn();
                }
            }
            // (A welcome may be from another server, after logging out.)
            ServerMessage::Welcome { .. }
            | ServerMessage::EnterLevel(_)
            | ServerMessage::OnWorldMap { .. }
            | ServerMessage::Teleported { .. } => {
                for (_, old) in std::mem::take(&mut bags.0) {
                    commands.entity(old.root).despawn();
                }
            }
            _ => {}
        }
    }
}
