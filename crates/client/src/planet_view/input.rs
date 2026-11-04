//! While open, the view has all input: any click, tap or Esc closes it, and
//! nothing behind it sees any of it.

use bevy::input::mouse::AccumulatedMouseScroll;
use bevy::input::touch::Touches;
use bevy::prelude::*;

use super::close::close;
use super::open::PlanetView;
use crate::common::TouchGesture;

/// Presses this soon after opening are left alone: the press that chose View
/// Planet in the debug menu mustn't close it again.
const SETTLING_SECONDS: f64 = 0.25;

/// After the pointer has been read (taps are turned into clicks then), before
/// the widgets, menus and maps look at anything.
#[allow(clippy::too_many_arguments)]
pub(super) fn take_input(
    mut view: ResMut<PlanetView>,
    mut keys: ResMut<ButtonInput<KeyCode>>,
    mut mouse: ResMut<ButtonInput<MouseButton>>,
    touches: Res<Touches>,
    mut scroll: ResMut<AccumulatedMouseScroll>,
    mut gesture: ResMut<TouchGesture>,
    time: Res<Time<Real>>,
    mut commands: Commands,
) {
    let Some(showing) = &view.showing else {
        return;
    };
    let settled = time.elapsed_secs_f64() - showing.opened > SETTLING_SECONDS;
    let pressed = keys.just_pressed(KeyCode::Escape)
        || mouse.get_just_pressed().next().is_some()
        || touches.any_just_pressed();
    if settled && pressed {
        close(&mut view, &mut commands);
    }
    keys.reset_all();
    mouse.reset_all();
    scroll.delta = Vec2::ZERO;
    *gesture = TouchGesture::default();
}
