//! Holding the minimap (the mouse button, or a finger) puts the camera back to
//! its starting view, as the Reset View key does (owner's choice, 2026-11-09,
//! for screens with no Home key). A short click or tap still opens the full map,
//! which now opens when it is let go.

use bevy::prelude::*;
use bevy::window::PrimaryWindow;

use super::{CORNER_GAP, FullMap, Room, minimap_across, minimap_picture_size};
use crate::common::{GameView, TouchGesture, UiPointer, in_interface};
use crate::view3d::OrbitCamera;

/// How long the minimap is held before the view goes back, in seconds: as a
/// long press targets (`target`).
const HOLD_SECONDS: f32 = 0.5;

/// A press on the minimap being held.
#[derive(Resource, Default)]
pub(super) struct MinimapHold {
    /// How long it has been held, while the mouse or a finger is on it.
    held: Option<f32>,
    /// It already put the view back, so letting go doesn't open the map.
    done: bool,
}

/// Whether this point (in interface pixels) is on the minimap.
fn on_minimap(window: &Room, at: Vec2) -> bool {
    let middle = Vec2::new(
        window.width() - CORNER_GAP - minimap_picture_size(window) / 2.0,
        CORNER_GAP + minimap_picture_size(window) / 2.0,
    );
    at.distance(middle) <= minimap_across(window) / 2.0
}

/// Takes presses on the minimap before the maps' `take_input` sees them: a
/// press held long enough puts every camera back; one let go before that
/// opens the full map.
#[allow(clippy::too_many_arguments)]
pub(super) fn hold_minimap(
    mut mouse: ResMut<ButtonInput<MouseButton>>,
    mut gesture: ResMut<TouchGesture>,
    window: Single<&Window, With<PrimaryWindow>>,
    ui_scale: Res<UiScale>,
    view: Res<State<GameView>>,
    pointer: Res<UiPointer>,
    time: Res<Time<Real>>,
    mut full: ResMut<FullMap>,
    mut hold: ResMut<MinimapHold>,
    mut cameras: Query<&mut OrbitCamera>,
) {
    if full.open || *view.get() == GameView::Connecting {
        *hold = MinimapHold::default();
        return;
    }
    let window = Room::of(&window, &ui_scale);
    let finger = gesture
        .still_at
        .map(|at| in_interface(at, &ui_scale))
        .filter(|&at| on_minimap(&window, at));
    let over = pointer.0.is_some_and(|at| on_minimap(&window, at));
    // A tap is a click on lifting the finger: the hold already decided it.
    if gesture.tap && hold.done {
        mouse.clear_just_pressed(MouseButton::Left);
        *hold = MinimapHold::default();
        return;
    }
    if over && mouse.clear_just_pressed(MouseButton::Left) {
        hold.held = Some(0.0);
        hold.done = false;
    }
    let mouse_held = hold.held.is_some() && mouse.pressed(MouseButton::Left);
    if finger.is_some() {
        // Not a long press for targeting as well.
        gesture.still_at = None;
        hold.held = Some(hold.held.unwrap_or(0.0));
    }
    let Some(held) = hold.held else {
        return;
    };
    if mouse_held || finger.is_some() {
        let held = held + time.delta_secs();
        hold.held = Some(held);
        if held >= HOLD_SECONDS && !hold.done {
            hold.done = true;
            for mut camera in &mut cameras {
                camera.reset();
            }
        }
        return;
    }
    // Let go: a short press opens the full map. A long one stays done until
    // the tap a lifted finger makes, or the next press.
    let done = hold.done;
    *hold = MinimapHold { held: None, done };
    if !done {
        full.set_open(true);
    }
}
