//! Whether the player is on the move, for others to give way to.

use bevy::prelude::*;

use super::{Player, PlayerEnergy, Travel};

/// Whether the player is moving: on a journey with some way still to go (or
/// stopped on one for a while), with steps waiting for energy, or holding a
/// movement key. `abilities` holds automatic ones back meanwhile, so a click
/// or a key always gets its way first (owner, 2026-11-07).
#[derive(Resource, Default, Debug)]
pub struct PlayerMoving {
    /// A movement key is held (`move_player`).
    pub(super) keys: bool,
    /// A journey, a jump or steps are under way (`note_moving`).
    pub(super) under_way: bool,
}

impl PlayerMoving {
    pub fn is_moving(&self) -> bool {
        self.keys || self.under_way
    }
}

/// After travelling: whether anything is still under way.
pub(super) fn note_moving(
    energy: Res<PlayerEnergy>,
    travel: Single<&Travel, With<Player>>,
    mut moving: ResMut<PlayerMoving>,
) {
    let under_way = !travel.route.is_empty() || travel.jump.is_some() || !energy.waiting.is_empty();
    // Only when it changes, so nothing is told it did when it didn't.
    if moving.under_way != under_way {
        moving.under_way = under_way;
    }
}

#[cfg(test)]
mod tests;
