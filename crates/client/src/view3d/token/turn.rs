//! Turning creatures' models towards the way their owners face
//! (`common::Facing`), smoothly, or at once with Reduce Motion. A model
//! whose owner faces no way turns to the camera, as the icons do.

use std::f32::consts::{FRAC_PI_2, PI, TAU};

use bevy::prelude::*;

use super::{ModelAnchor, TokenAssets, TokenView};
use crate::common::{Facing, Preferences};
use crate::view3d::ViewCamera;

/// How fast a model turns towards its heading, at most: radians a second
/// (about two turns a second, so a quick turn still reads as a turn).
const TURN_SPEED: f32 = 4.0 * PI;

/// The models' anchors' rotation about the vertical: towards their owner's
/// heading, from where it was shown last frame.
pub fn turn_models(
    time: Res<Time>,
    preferences: Res<Preferences>,
    camera: Single<&Transform, (With<ViewCamera>, Without<ModelAnchor>)>,
    views: Query<&TokenView>,
    facings: Query<&Facing>,
    mut anchors: Query<(&mut ModelAnchor, &ChildOf, &mut Transform)>,
) {
    let snap = preferences.graphics.reduce_motion;
    let most = TURN_SPEED * time.delta_secs();
    let to_camera = TokenAssets::icon_rotation(camera.rotation);
    for (mut anchor, child_of, mut transform) in &mut anchors {
        let facing = views
            .get(child_of.parent())
            .ok()
            .and_then(|view| facings.get(view.owner).ok());
        let Some(facing) = facing else {
            anchor.angle = None;
            transform.rotation = to_camera;
            continue;
        };
        // The model's front (+z) along the heading: east is +x, south +z.
        let wanted = FRAC_PI_2 - facing.0.radians();
        let shown = match anchor.angle {
            Some(was) if !snap => was + shortest(wanted - was).clamp(-most, most),
            _ => wanted,
        };
        anchor.angle = Some(shown.rem_euclid(TAU));
        transform.rotation = Quat::from_rotation_y(shown);
    }
}

/// An angle brought into -π to π: the short way round.
fn shortest(angle: f32) -> f32 {
    (angle + PI).rem_euclid(TAU) - PI
}
