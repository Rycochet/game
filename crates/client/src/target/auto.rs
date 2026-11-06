//! Targets chosen for the player: an enemy coming up to them, and the next
//! enemy when the target dies.

use bevy::prelude::*;
use shared::protocol::ServerMessage;

use super::choose::{enemy_beside, nearest_enemy};
use super::seen::seen_now;
use crate::common::{MapPosition, Other, Target};
use crate::connection::FromServer;
use crate::player::Player;

/// With no target in sight, the first enemy to come right up to the player
/// becomes it. (Once there is combat, whoever attacks them is to.)
pub(super) fn pick_enemy_beside(
    player: Single<&MapPosition, With<Player>>,
    others: Query<(&Other, &MapPosition)>,
    mut target: ResMut<Target>,
) {
    let seen = seen_now(&others);
    let in_sight = target
        .0
        .is_some_and(|id| seen.iter().any(|one| one.id == id));
    if in_sight {
        return;
    }
    if let Some(enemy) = enemy_beside((player.x, player.y), &seen) {
        target.set_if_neq(Target(Some(enemy)));
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
