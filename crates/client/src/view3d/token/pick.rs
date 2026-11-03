//! The token clicked or under the cursor.

use bevy::prelude::*;

use super::super::ViewCamera;
use super::{DIAMETER, ICON_ART_SIZE, IconAnchor, TokenView};
use crate::common::{Clickable, PointerPosition};

/// The `Clickable` token the left mouse button was just pressed on, if any
/// (nearest the camera when several overlap). Set every frame in
/// `View3dSystems::Click`; views ignore a click on the ground under it.
#[derive(Resource, Default, Clone, Copy, Debug, PartialEq, Eq)]
pub struct ClickedToken(pub Option<Entity>);

/// The clickable token the mouse is over (nearest the camera when several
/// overlap), if any. Set every frame in `View3dSystems::Click`.
#[derive(Resource, Default, Clone, Copy, Debug, PartialEq, Eq)]
pub struct HoveredToken(pub Option<Entity>);

/// Finds the clickable token under the cursor: within the token's circle on
/// screen, or its icon's main picture. A left click on it is `ClickedToken`.
#[allow(clippy::too_many_arguments)]
pub fn find_token_under_cursor(
    mouse: Res<ButtonInput<MouseButton>>,
    pointer: Res<PointerPosition>,
    camera: Single<(&Camera, &GlobalTransform), With<ViewCamera>>,
    views: Query<(&TokenView, &GlobalTransform, &InheritedVisibility)>,
    anchors: Query<(&IconAnchor, &GlobalTransform)>,
    clickable: Query<(), With<Clickable>>,
    mut hovered: ResMut<HoveredToken>,
    mut clicked: ResMut<ClickedToken>,
) {
    let found = pointer.0.and_then(|cursor| {
        let (camera, camera_transform) = *camera;
        token_at(
            cursor,
            camera,
            camera_transform,
            &views,
            &anchors,
            &clickable,
        )
    });
    hovered.set_if_neq(HoveredToken(found));
    let pressed = mouse.just_pressed(MouseButton::Left);
    clicked.set_if_neq(ClickedToken(found.filter(|_| pressed)));
}

/// The clickable token at a point on the screen.
pub(super) fn token_at(
    cursor: Vec2,
    camera: &Camera,
    camera_transform: &GlobalTransform,
    views: &Query<(&TokenView, &GlobalTransform, &InheritedVisibility)>,
    anchors: &Query<(&IconAnchor, &GlobalTransform)>,
    clickable: &Query<(), With<Clickable>>,
) -> Option<Entity> {
    let right = camera_transform.right();
    let eye = camera_transform.translation();
    // Whether the cursor is within `half` (3D units, across the screen) of a
    // point: as a circle or a square.
    let near = |point: Vec3, half: f32, square: bool| {
        let (Ok(centre), Ok(side)) = (
            camera.world_to_viewport(camera_transform, point),
            camera.world_to_viewport(camera_transform, point + right * half),
        ) else {
            return false;
        };
        let reach = centre.distance(side);
        let offset = cursor - centre;
        if square {
            offset.x.abs() <= reach && offset.y.abs() <= reach
        } else {
            offset.length() <= reach
        }
    };

    let mut best: Option<(f32, Entity)> = None;
    for (view, transform, visibility) in views {
        if !visibility.get() || clickable.get(view.owner).is_err() {
            continue;
        }
        let base = transform.translation();
        let mut hit = near(base, DIAMETER / 2.0, false);
        if let Ok((anchor, anchor_transform)) = anchors.get(view.anchor) {
            hit |= anchor.shown.is_some()
                && near(
                    anchor_transform.translation(),
                    ICON_ART_SIZE * anchor.scale / 2.0,
                    true,
                );
        }
        let distance = eye.distance(base);
        if hit && best.is_none_or(|(d, _)| distance < d) {
            best = Some((distance, view.owner));
        }
    }
    best.map(|(_, owner)| owner)
}
