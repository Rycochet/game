//! Steps asked for while a key is held.

use bevy::prelude::*;
use shared::movement::Step;

use super::{PlayerEnergy, TakeStep};
use crate::common::ViewFacing;

/// Asks for a step while a key is held, as soon as the last one has gone: energy
/// sets the pace.
pub(super) fn move_player(
    keys: Res<ButtonInput<KeyCode>>,
    facing: Res<ViewFacing>,
    energy: Res<PlayerEnergy>,
    mut steps: MessageWriter<TakeStep>,
) {
    // A key tapped and released within one frame is "just pressed" but no
    // longer "pressed", so count either or quick taps get lost.
    let active = |a, b| keys.any_pressed([a, b]) || keys.any_just_pressed([a, b]);
    let mut dx = 0;
    let mut dy = 0;
    if active(KeyCode::ArrowLeft, KeyCode::KeyA) {
        dx -= 1;
    }
    if active(KeyCode::ArrowRight, KeyCode::KeyD) {
        dx += 1;
    }
    if active(KeyCode::ArrowUp, KeyCode::KeyW) {
        dy -= 1;
    }
    if active(KeyCode::ArrowDown, KeyCode::KeyS) {
        dy += 1;
    }

    if (dx != 0 || dy != 0) && energy.waiting.is_empty() {
        // The keys mean screen directions, so follow the way the map is turned.
        let (dx, dy) = facing.screen_to_grid(dx, dy);
        if let Some(step) = Step::new(dx, dy) {
            steps.write(TakeStep(step));
        }
    }
}
