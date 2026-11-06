//! Changing the target with keys: Tab outwards, Shift+Tab inwards.

use bevy::prelude::*;

use super::choose::{cycle, in_order};
use super::seen::seen_now;
use crate::common::{Action, MapPosition, Other, Preferences, Target};
use crate::player::Player;

/// Next Target (Tab) picks the next creature further out than the target, or
/// the nearest without one; Previous Target (Shift+Tab) the next in. Only
/// enemies, unless none are in sight.
pub(super) fn cycle_by_keys(
    keys: Res<ButtonInput<KeyCode>>,
    preferences: Res<Preferences>,
    player: Single<&MapPosition, With<Player>>,
    others: Query<(&Other, &MapPosition)>,
    mut target: ResMut<Target>,
) {
    let controls = &preferences.controls;
    let outwards = match (
        controls.just_pressed(&keys, Action::NextTarget),
        controls.just_pressed(&keys, Action::PreviousTarget),
    ) {
        (true, false) => true,
        (false, true) => false,
        _ => return,
    };
    let order = in_order((player.x, player.y), &seen_now(&others));
    if let Some(next) = cycle(&order, target.0, outwards) {
        target.set_if_neq(Target(Some(next)));
    }
}
