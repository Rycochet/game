//! Steps asked for while a key, or an arrow on the movement pad, is held.

use bevy::prelude::*;
use shared::movement::Step;

use super::{PlayerEnergy, PlayerMoving, TakeStep};
use crate::common::{Action, PadSteps, Preferences, ViewFacing};

/// Asks for a step while a key (or a pad arrow, `PadSteps`) is held, as soon as the last one has gone: energy
/// sets the pace. The first step of a hold is always taken; once it has been,
/// letting go drops the one waiting, so a click or tap lasting a few frames is
/// one step.
pub(super) fn move_player(
    (keys, preferences): (Res<ButtonInput<KeyCode>>, Res<Preferences>),
    facing: Res<ViewFacing>,
    mut energy: ResMut<PlayerEnergy>,
    pad: Option<Res<PadSteps>>,
    mut moving: ResMut<PlayerMoving>,
    mut steps: MessageWriter<TakeStep>,
    mut asked: Local<u32>,
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

    if let Some(pad) = pad {
        dx = (dx + pad.dx).clamp(-1, 1);
        dy = (dy + pad.dy).clamp(-1, 1);
    }

    let held = dx != 0 || dy != 0;
    if moving.keys != held {
        moving.keys = held;
    }
    if !held {
        // More than one asked: the first has been taken, as the next was only
        // asked once it had gone.
        if *asked > 1 {
            energy.waiting.clear();
        }
        *asked = 0;
    }
    if held && energy.waiting.is_empty() {
        // The keys mean screen directions, so follow the way the map is turned.
        let (dx, dy) = facing.screen_to_grid(dx, dy);
        if let Some(step) = Step::new(dx, dy) {
            steps.write(TakeStep(step));
            *asked += 1;
        }
    }
}

#[cfg(test)]
mod tests;
