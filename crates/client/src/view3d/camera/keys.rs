//! The camera's keys: Turn Camera Left and Right, Zoom In and Out. A press
//! moves it a small step at once, and holding the key keeps it going at a
//! steady speed. Each frame looks only at whether the key is down now, so
//! nothing builds up: it stops the moment the key comes up.

use bevy::prelude::*;

use super::OrbitCamera;
use crate::common::{Action, Controls, Preferences};

/// How far a press turns the camera at once.
const TURN_STEP_DEGREES: f32 = 5.0;
/// How fast it keeps turning while the key is held.
const TURN_DEGREES_PER_SECOND: f32 = 60.0;
/// How far a press zooms at once, in mouse wheel notches.
const ZOOM_STEP_NOTCHES: f32 = 1.0;
/// How fast it keeps zooming while the key is held.
const ZOOM_NOTCHES_PER_SECOND: f32 = 3.0;

/// How a step and a speed apply to one action this frame.
#[derive(Clone, Copy)]
struct Pace {
    step: f32,
    per_second: f32,
}

const TURN: Pace = Pace {
    step: TURN_STEP_DEGREES,
    per_second: TURN_DEGREES_PER_SECOND,
};
const ZOOM: Pace = Pace {
    step: ZOOM_STEP_NOTCHES,
    per_second: ZOOM_NOTCHES_PER_SECOND,
};

/// How far `more` less `less` moves things this frame: the step on the
/// frame a key goes down, then the speed for as long as it stays down.
fn amount(
    controls: &Controls,
    keys: &ButtonInput<KeyCode>,
    (less, more): (Action, Action),
    pace: Pace,
    seconds: f32,
) -> f32 {
    let way = |action| {
        if controls.just_pressed(keys, action) {
            pace.step
        } else if controls.pressed(keys, action) {
            pace.per_second * seconds
        } else {
            0.0
        }
    };
    way(more) - way(less)
}

/// Turns and zooms every `OrbitCamera` by the camera keys. Real time, as the
/// mouse is: a paused game still lets the camera move.
pub fn control_with_keys(
    keys: Res<ButtonInput<KeyCode>>,
    preferences: Res<Preferences>,
    time: Res<Time<Real>>,
    mut cameras: Query<&mut OrbitCamera>,
) {
    let controls = &preferences.controls;
    let seconds = time.delta_secs();
    // Turning right moves the camera east of the player: yaw goes up.
    let turn = amount(
        controls,
        &keys,
        (Action::TurnLeft, Action::TurnRight),
        TURN,
        seconds,
    );
    let zoom = amount(
        controls,
        &keys,
        (Action::ZoomOut, Action::ZoomIn),
        ZOOM,
        seconds,
    );
    if turn == 0.0 && zoom == 0.0 {
        // Leave the cameras untouched, so their change detection stays quiet.
        return;
    }
    for mut camera in &mut cameras {
        camera.twist(turn.to_radians());
        camera.zoom_by(zoom);
    }
}

#[cfg(test)]
mod tests;
