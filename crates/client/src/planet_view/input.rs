//! While open, the view has all input. The bar's controls get theirs first, as
//! widgets do; the camera's controls are read (`steer`); a click or tap goes to
//! a gate's ring or closes it, and so does Esc; and nothing behind it sees any
//! of it.

use bevy::input::mouse::AccumulatedMouseScroll;
use bevy::prelude::*;

use super::close::close;
use super::showing::PlanetView;
use crate::common::TouchGesture;

/// Presses this soon after opening are left alone: the press that chose View
/// Planet in the debug menu mustn't close it again.
const SETTLING_SECONDS: f64 = 0.25;

/// After the widgets have taken presses on the bar (and an open dropdown or the
/// seed box its keys), before the system menu or anything behind looks at what
/// is left.
#[allow(clippy::too_many_arguments)]
pub(super) fn take_input(
    mut view: ResMut<PlanetView>,
    mut keys: ResMut<ButtonInput<KeyCode>>,
    mut mouse: ResMut<ButtonInput<MouseButton>>,
    mut scroll: ResMut<AccumulatedMouseScroll>,
    mut gesture: ResMut<TouchGesture>,
    time: Res<Time<Real>>,
    mut commands: Commands,
) {
    let Some(showing) = &view.showing else {
        return;
    };
    let settled = time.elapsed_secs_f64() - showing.opened > SETTLING_SECONDS;
    // A tap arrives as a click, which the bar's widgets have taken if it was
    // theirs; one elsewhere still shows here.
    // The right button turns the camera (`steer`), so only the left closes.
    let clicked = mouse.just_pressed(MouseButton::Left);
    // The click that takes the keyboard from the seed box only does that.
    // A click goes to a gate's ring under it, or closes the view, once the
    // ring is known (`teleport::answer_click`).
    let clicked = clicked && !showing.typing;
    if settled && keys.just_pressed(KeyCode::Escape) {
        close(&mut view, &mut commands);
    } else if settled
        && clicked
        && let Some(showing) = view.showing.as_mut()
    {
        showing.clicked = true;
    }
    keys.reset_all();
    mouse.reset_all();
    scroll.delta = Vec2::ZERO;
    *gesture = TouchGesture::default();
}
