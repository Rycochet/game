//! The token clicked or under the cursor.

use bevy::prelude::*;

use super::super::ViewCamera;
use super::{ICON_ART_SIZE, IconAnchor, MODEL_SCALE, ModelAnchor, TokenView};
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
    views: Views,
    anchors: Anchors,
    models: Models,
    clickable: Query<(), With<Clickable>>,
    named: Query<(), With<HoverName>>,
    mut hovered: ResMut<HoveredToken>,
    mut clicked: ResMut<ClickedToken>,
) {
    let (camera, camera_transform) = *camera;
    let find = |wanted: &dyn Fn(Entity) -> bool| {
        pointer.0.and_then(|cursor| {
            let shown = Shown {
                views: &views,
                anchors: &anchors,
                models: &models,
            };
            token_at(cursor, camera, camera_transform, &shown, wanted)
        })
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

type Views<'w, 's> = Query<
    'w,
    's,
    (
        &'static TokenView,
        &'static GlobalTransform,
        &'static InheritedVisibility,
    ),
>;
type Anchors<'w, 's> = Query<'w, 's, (&'static IconAnchor, &'static GlobalTransform)>;
type Models<'w, 's> = Query<'w, 's, (&'static ModelAnchor, &'static GlobalTransform)>;

/// What is drawn of tokens, to find one under the pointer.
pub(super) struct Shown<'a, 'w, 's> {
    pub(super) views: &'a Views<'w, 's>,
    pub(super) anchors: &'a Anchors<'w, 's>,
    pub(super) models: &'a Models<'w, 's>,
}

/// The nearest token at a point on the screen whose owner is `wanted`: on its base,
/// its icon, or its model's upright box.
pub(super) fn token_at(
    cursor: Vec2,
    camera: &Camera,
    camera_transform: &GlobalTransform,
    shown: &Shown,
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
    // Whether the cursor is on a box standing on `feet`, `half` across and `tall`.
    let on_box = |feet: Vec3, half: f32, tall: f32| {
        let corners = [feet + right * half, feet - right * half + Vec3::Y * tall];
        let (Ok(a), Ok(b)) = (
            camera.world_to_viewport(camera_transform, corners[0]),
            camera.world_to_viewport(camera_transform, corners[1]),
        ) else {
            return false;
        };
        let (low, high) = (a.min(b), a.max(b));
        cursor.cmpge(low).all() && cursor.cmple(high).all()
    };
    let (views, anchors) = (shown.views, shown.anchors);

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
        if let Ok((anchor, model_transform)) = shown.models.get(view.model)
            && let Some(model) = anchor.model()
        {
            let (half, tall) = (model.0.reach * MODEL_SCALE, model.0.height * MODEL_SCALE);
            hit |= on_box(model_transform.translation(), half, tall);
        }
        let distance = eye.distance(base);
        if hit && best.is_none_or(|(d, _)| distance < d) {
            best = Some((distance, view.owner));
        }
    }
    best.map(|(_, owner)| owner)
}
