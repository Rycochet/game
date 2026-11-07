//! Tokens flash red when what they stand for is harmed: the player's own,
//! their target struck by them, or anyone else in sight (`ServerMessage::Harmed`).

use bevy::prelude::*;
use shared::id::Id;
use shared::protocol::{Landed, ServerMessage};
use shared::vitals::Points;

use super::Others;
use crate::connection::FromServer;
use crate::player::Player;
use crate::view3d::TokenFlash;

/// Whose token flashes for a message.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum Harmed {
    /// The player.
    Player,
    /// Someone else in sight.
    Other(Id),
}

/// Who a message says was harmed, if anyone. A blow that did no harm
/// (stopped by armour) doesn't count.
pub(super) fn harmed(message: &ServerMessage) -> Option<Harmed> {
    match message {
        ServerMessage::AbilityLanded {
            target,
            landed: Landed::Hit { harm, .. },
            ..
        } if *harm != Points::ZERO => Some(match target {
            // Aimed at the player.
            None => Harmed::Player,
            Some(id) => Harmed::Other(*id),
        }),
        ServerMessage::Harmed { id, .. } => Some(Harmed::Other(*id)),
        _ => None,
    }
}

/// Starts a flash on the token of anyone harmed.
pub(super) fn flash_harmed(
    mut from_server: MessageReader<FromServer>,
    others: Res<Others>,
    player: Query<Entity, With<Player>>,
    mut commands: Commands,
) {
    for FromServer(message) in from_server.read() {
        let owner = match harmed(message) {
            Some(Harmed::Player) => player.single().ok(),
            Some(Harmed::Other(id)) => others.0.get(&id).copied(),
            None => None,
        };
        if let Some(owner) = owner
            && let Ok(mut owner) = commands.get_entity(owner)
        {
            owner.try_insert(TokenFlash::default());
        }
    }
}

#[cfg(test)]
mod tests;
