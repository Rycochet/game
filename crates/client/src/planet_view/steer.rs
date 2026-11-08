//! Moving the view's camera round the globe by the maps' controls: right-drag
//! turns and tilts, the wheel zooms, two fingers pinch, twist and tilt, the
//! camera keys turn and zoom, and Reset View (Home) puts it back. Read before
//! `take_input` keeps it all from the rest of the game.

use bevy::input::mouse::{AccumulatedMouseMotion, AccumulatedMouseScroll, MouseScrollUnit};
use bevy::prelude::*;

use super::showing::PlanetView;
use crate::common::{Action, Controls, Preferences, TouchGesture};

/// Touchpads scroll in pixels rather than notches; this many make one notch,
/// as on the maps.
const PIXELS_PER_NOTCH: f32 = 100.0;
/// A camera key's press moves this far at once, and holding it goes on at
/// this speed, as on the maps: degrees for turning, notches for zooming.
const TURN: Pace = Pace {
    step: 5.0,
    per_second: 60.0,
};
const ZOOM: Pace = Pace {
    step: 1.0,
    per_second: 3.0,
};

#[derive(Clone, Copy)]
struct Pace {
    step: f32,
    per_second: f32,
}

#[allow(clippy::too_many_arguments)]
pub(super) fn steer(
    mut view: ResMut<PlanetView>,
    buttons: Res<ButtonInput<MouseButton>>,
    motion: Res<AccumulatedMouseMotion>,
    scroll: Res<AccumulatedMouseScroll>,
    keys: Res<ButtonInput<KeyCode>>,
    preferences: Res<Preferences>,
    gesture: Res<TouchGesture>,
    time: Res<Time<Real>>,
) {
    let Some(showing) = view.showing.as_mut() else {
        return;
    };
    let start = showing.globe.scene.distance;
    let orbit = &mut showing.orbit;
    if buttons.pressed(MouseButton::Right) && motion.delta != Vec2::ZERO {
        orbit.turn(motion.delta);
    }
    let notches = match scroll.unit {
        MouseScrollUnit::Line => scroll.delta.y,
        MouseScrollUnit::Pixel => scroll.delta.y / PIXELS_PER_NOTCH,
    };
    if notches != 0.0 {
        orbit.zoom_by(notches, start);
    }
    if gesture.pinch != 1.0 {
        orbit.zoom_by_pinch(gesture.pinch, start);
    }
    if gesture.twist != 0.0 || gesture.tilt != 0.0 {
        orbit.twist(gesture.twist);
        orbit.turn(Vec2::new(0.0, gesture.tilt));
    }
    // The seed box has the keys while it is being typed in.
    if showing.typing {
        return;
    }
    let controls = &preferences.controls;
    let seconds = time.delta_secs();
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
    orbit.twist(turn.to_radians());
    if zoom != 0.0 {
        orbit.zoom_by(zoom, start);
    }
    if controls.just_pressed(&keys, Action::ResetView) {
        orbit.reset();
    }
}

/// How far `more` less `less` moves things this frame: the step on the frame
/// a key goes down, then the speed for as long as it stays down.
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
