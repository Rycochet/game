//! Steps asked for while a key is held.

use bevy::prelude::*;
use shared::movement::Step;

use super::{PlayerEnergy, PlayerMoving, TakeStep};
use crate::common::{Action, Preferences, ViewFacing};

/// Asks for a step while a key is held, as soon as the last one has gone: energy
/// sets the pace.
pub(super) fn move_player(
    keys: Res<ButtonInput<KeyCode>>,
    preferences: Res<Preferences>,
    facing: Res<ViewFacing>,
    energy: Res<PlayerEnergy>,
    mut moving: ResMut<PlayerMoving>,
    mut steps: MessageWriter<TakeStep>,
) {
    // A key tapped and released within one frame is "just pressed" but no
    // longer "pressed": `active` counts either, or quick taps get lost.
    let active = |action| preferences.controls.active(&keys, action);
    let mut dx = 0;
    let mut dy = 0;
    if active(Action::StepLeft) {
        dx -= 1;
    }
    if active(Action::StepRight) {
        dx += 1;
    }
    if active(Action::StepUp) {
        dy -= 1;
    }
    if active(Action::StepDown) {
        dy += 1;
    }

    let held = dx != 0 || dy != 0;
    if moving.keys != held {
        moving.keys = held;
    }
    if held && energy.waiting.is_empty() {
        // The keys mean screen directions, so follow the way the map is turned.
        let (dx, dy) = facing.screen_to_grid(dx, dy);
        if let Some(step) = Step::new(dx, dy) {
            steps.write(TakeStep(step));
        }
    }
}
