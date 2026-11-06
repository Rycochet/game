//! Targeting creatures: a right click (or a long press) on one, Tab and
//! Shift+Tab to go through them from nearest to furthest, and targets chosen
//! for the player; a red reticule turning on the target's token. See
//! `docs/context/client/target.md`.

mod auto;
mod choose;
mod keys;
mod pick;
mod reticule;
mod seen;

use bevy::prelude::*;
use bevy::transform::TransformSystems;

use crate::common::GameView;
use crate::player::PlayerSystems;
use crate::view3d::View3dSystems;

/// Targeting, and the reticule.
pub struct TargetPlugin;

impl Plugin for TargetPlugin {
    fn build(&self, app: &mut App) {
        let playing = not(in_state(GameView::Connecting));
        app.init_resource::<pick::LongPress>()
            .add_systems(Startup, reticule::spawn_reticule)
            .add_systems(
                Update,
                pick::eat_tap
                    .before(PlayerSystems::Input)
                    .run_if(playing.clone()),
            )
            .add_systems(
                Update,
                (
                    pick::pick_by_right_click,
                    pick::pick_by_long_press,
                    keys::cycle_by_keys,
                    auto::pick_after_death,
                    auto::pick_enemy_beside,
                )
                    .chain()
                    .after(View3dSystems::Click)
                    .run_if(playing),
            )
            .add_systems(
                PostUpdate,
                reticule::place_reticule.before(TransformSystems::Propagate),
            );
    }
}

#[cfg(test)]
mod tests;
