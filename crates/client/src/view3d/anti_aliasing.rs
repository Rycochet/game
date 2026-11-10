//! Anti-aliasing as chosen, on the view's camera. Without it, faded tokens'
//! `AlphaToCoverage` becomes a plain cut-out (Bevy does this itself).

use bevy::prelude::*;

use super::ViewCamera;
use crate::common::{AntiAliasing, Preferences};

pub(super) fn apply_anti_aliasing(
    preferences: Res<Preferences>,
    camera: Single<&mut Msaa, With<ViewCamera>>,
) {
    if !preferences.is_changed() {
        return;
    }
    let wanted = match preferences.graphics.anti_aliasing.in_use() {
        AntiAliasing::Off => Msaa::Off,
        AntiAliasing::Times2 => Msaa::Sample2,
        AntiAliasing::Times4 => Msaa::Sample4,
    };
    let mut msaa = camera.into_inner();
    if *msaa != wanted {
        *msaa = wanted;
    }
}
