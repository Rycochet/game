//! A drawn token's root and the point its icon hangs from.

use bevy::prelude::*;

/// The root of a drawn token. It never rotates, so children can rely on
/// world axes. `owner` is the entity with the `Token`.
#[derive(Component)]
pub struct TokenView {
    pub owner: Entity,
    pub(super) body: Entity,
    pub(super) outline: Entity,
    /// The body and outline drawn through whatever hides them (`xray`).
    pub(super) xray_body: Entity,
    pub(super) xray_outline: Entity,
    pub(super) anchor: Entity,
}

/// A point above each token that is turned to face the camera every frame.
/// The icon is its child: a flat quad in its XY plane, so it always shows the
/// same face to the camera, however the map is rotated. `shown` is the icon
/// path currently displayed; `scale` the owner's `TokenSize` scale.
#[derive(Component)]
pub struct IconAnchor {
    pub(super) shown: Option<String>,
    pub(super) scale: f32,
}

impl IconAnchor {
    /// How big the icon is drawn, relative to `TokenSize::Default`.
    pub fn scale(&self) -> f32 {
        self.scale
    }
}
