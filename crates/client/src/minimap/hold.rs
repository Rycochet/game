//! Holding the minimap (the mouse button, or a finger) puts the camera back to
//! its starting view, as the Reset View key does (owner's choice, 2026-11-09,
//! for screens with no Home key). A short click or tap still opens the full map,
//! which now opens when it is let go.

use bevy::prelude::*;

use super::{FullMap, MapPart};
use crate::common::{GameView, TouchGesture, in_interface};
use crate::dom::{Finder, Hit};
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

/// Takes presses on the minimap before the maps' `take_input` sees them: a
/// press held long enough puts every camera back; one let go before that
/// opens the full map. Whether the mouse or a finger is on the minimap's
/// face is `dom`'s search (`Hit`, `Finder`), which leaves the press here
/// (`dom::OwnPointer`).
#[allow(clippy::too_many_arguments)]
pub(super) fn hold_minimap(
    mut mouse: ResMut<ButtonInput<MouseButton>>,
    mut gesture: ResMut<TouchGesture>,
    ui_scale: Res<UiScale>,
    view: Res<State<GameView>>,
    (hit, finder, parts): (Res<Hit>, Finder, Query<&MapPart>),
    time: Res<Time<Real>>,
    mut full: ResMut<FullMap>,
    mut hold: ResMut<MinimapHold>,
    mut cameras: Query<&mut OrbitCamera>,
) {
    if full.open || *view.get() == GameView::Connecting {
        *hold = MinimapHold::default();
        return;
    }
    let on_minimap = |chain: &[Entity]| MapPart::Minimap.holds(chain, &parts);
    let finger = gesture
        .still_at
        .map(|at| in_interface(at, &ui_scale))
        .filter(|&at| on_minimap(&finder.chain_at(at)));
    let over = on_minimap(&hit.chain);
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
