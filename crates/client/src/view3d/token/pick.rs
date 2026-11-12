//! The token clicked or under the cursor.

use bevy::prelude::*;

use super::super::ViewCamera;
use super::{ICON_ART_SIZE, IconAnchor, TokenView};
use crate::common::{Clickable, HoverName, PointerPosition};

/// The `Clickable` token the left mouse button was just pressed on, if any
/// (nearest the camera when several overlap). Set every frame in
/// `View3dSystems::Click`; views ignore a click on the ground under it.
#[derive(Resource, Default, Clone, Copy, Debug, PartialEq, Eq)]
pub struct ClickedToken(pub Option<Entity>);

/// The token the mouse is over that can be clicked or has a `HoverName`
/// (nearest the camera when several overlap), if any. Set every frame in
/// `View3dSystems::Click`.
#[derive(Resource, Default, Clone, Copy, Debug, PartialEq, Eq)]
pub struct HoveredToken(pub Option<Entity>);

/// Finds the token under the cursor: within the token's circle on screen, or
/// its icon's main picture. One with a name is hovered even if it can't be
/// clicked; a left click is `ClickedToken` only on a clickable one.
#[allow(clippy::too_many_arguments)]
pub fn find_token_under_cursor(
    mouse: Res<ButtonInput<MouseButton>>,
    pointer: Res<PointerPosition>,
    camera: Single<(&Camera, &GlobalTransform), With<ViewCamera>>,
    views: Query<(&TokenView, &GlobalTransform, &InheritedVisibility)>,
    anchors: Query<(&IconAnchor, &GlobalTransform)>,
    clickable: Query<(), With<Clickable>>,
    named: Query<(), With<HoverName>>,
    mut hovered: ResMut<HoveredToken>,
    mut clicked: ResMut<ClickedToken>,
) {
    let (camera, camera_transform) = *camera;
    let find = |wanted: &dyn Fn(Entity) -> bool| {
        pointer
            .0
            .and_then(|cursor| token_at(cursor, camera, camera_transform, &views, &anchors, wanted))
    };
    let found = find(&|owner| clickable.contains(owner) || named.contains(owner));
    hovered.set_if_neq(HoveredToken(found));
    // A creature that can't be clicked may stand in front of one that can.
    let pressed = mouse.just_pressed(MouseButton::Left);
    let click = match found {
        _ if !pressed => None,
        Some(owner) if clickable.contains(owner) => Some(owner),
        Some(_) => find(&|owner| clickable.contains(owner)),
        None => None,
    };
    clicked.set_if_neq(ClickedToken(click));
}

/// The nearest token at a point on the screen whose owner is `wanted`.
pub(super) fn token_at(
    cursor: Vec2,
    camera: &Camera,
    camera_transform: &GlobalTransform,
    views: &Query<(&TokenView, &GlobalTransform, &InheritedVisibility)>,
    anchors: &Query<(&IconAnchor, &GlobalTransform)>,
    wanted: &dyn Fn(Entity) -> bool,
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
        if !visibility.get() || !wanted(view.owner) {
            continue;
        }
        let base = transform.translation();
        let mut hit = near(base, view.radius, false);
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
