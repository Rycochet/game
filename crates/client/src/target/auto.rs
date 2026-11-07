//! Targets chosen for the player: whoever attacks them, and the next enemy
//! when the target dies.

use bevy::prelude::*;
use shared::protocol::ServerMessage;

use super::choose::nearest_enemy;
use super::seen::seen_now;
use crate::common::{MapPosition, Other, Target};
use crate::connection::FromServer;
use crate::player::Player;

/// With no target in sight, whoever begins an attack on the player becomes it
/// (owner), so an automatic Attack answers them.
pub(super) fn pick_attacker(
    mut from_server: MessageReader<FromServer>,
    others: Query<(&Other, &MapPosition)>,
    mut target: ResMut<Target>,
) {
    for FromServer(message) in from_server.read() {
        let ServerMessage::AbilityStarted {
            who: Some(attacker),
            target: None,
            ..
        } = message
        else {
            continue;
        };
        let seen = seen_now(&others);
        let in_sight = target
            .0
            .is_some_and(|id| seen.iter().any(|one| one.id == id));
        if !in_sight {
            target.set_if_neq(Target(Some(*attacker)));
        }
    }
}

/// The target died in sight: the nearest other enemy in sight takes its place,
/// or nobody. A death out of sight is never told, so the target stays.
pub(super) fn pick_after_death(
    mut from_server: MessageReader<FromServer>,
    player: Single<&MapPosition, With<Player>>,
    others: Query<(&Other, &MapPosition)>,
    mut target: ResMut<Target>,
) {
    for FromServer(message) in from_server.read() {
        if let ServerMessage::EntityGone { id, died: true } = message
            && target.0 == Some(*id)
        {
            let next = nearest_enemy((player.x, player.y), &seen_now(&others), *id);
            target.set_if_neq(Target(next));
        }
    }
}
