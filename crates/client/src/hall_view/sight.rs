//! How the Hall of Worlds is seen: the camera goes `REACH` times as far out
//! as elsewhere, and tilts no lower than `LOWEST_PITCH`, so the top of the
//! screen always meets the floor within the stretch the client holds
//! (`site`'s, at least 75 tiles each way) and the Hall seems endless.

use bevy::prelude::*;

use super::in_hall;
use crate::common::{CurrentDungeon, GameView};
use crate::view3d::{OrbitCamera, ViewCamera};

/// How much further out the camera goes in the Hall (owner: about 25%).
const REACH: f32 = 1.25;
/// The lowest the camera tilts in the Hall, in degrees: at the furthest
/// zoom, the top of the screen meets the floor about 72 tiles past the
/// player. Lower needs many more tiles (infinitely many at 10 degrees, where
/// the top of the screen is level).
const LOWEST_PITCH: f32 = 18.0;

/// In the Hall, the further reach and the higher lowest tilt; anywhere else,
/// the usual.
pub(super) fn hall_reach(
    dungeon: Res<CurrentDungeon>,
    view: Res<State<GameView>>,
    mut camera: Query<&mut OrbitCamera, With<ViewCamera>>,
    mut was_here: Local<bool>,
) {
    let here = in_hall(&dungeon, *view.get());
    let (reach, lowest) = if here {
        (REACH, Some(LOWEST_PITCH))
    } else {
        (1.0, None)
    };
    let Ok(mut orbit) = camera.single_mut() else {
        return;
    };
    if orbit.reach() != reach {
        orbit.set_reach(reach);
    }
    // On coming in or going out.
    if here != *was_here {
        *was_here = here;
        orbit.set_lowest_pitch(lowest);
    }
}
