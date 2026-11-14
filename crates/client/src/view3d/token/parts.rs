//! A drawn token's root and the point its icon hangs from.

use bevy::prelude::*;

/// The root of a drawn token. It never rotates, so children can rely on
/// world axes; only its `base` tilts to the ground (`BaseTilt`). `owner` is
/// the entity with the `Token`.
#[derive(Component)]
pub struct TokenView {
    pub owner: Entity,
    /// The base's four meshes' parent, turned to lie on the ground.
    pub(in crate::view3d) base: Entity,
    pub(super) body: Entity,
    pub(super) outline: Entity,
    /// The body and outline drawn through whatever hides them (`xray`).
    pub(super) xray_body: Entity,
    pub(super) xray_outline: Entity,
    pub(in crate::view3d) anchor: Entity,
    /// Where a 3D model stands (`model`), in place of the icon.
    pub(in crate::view3d) model: Entity,
    /// The base's radius, in 3D units (`base_radius`), for clicks and tilt.
    pub(in crate::view3d) radius: f32,
    /// How the base is tilted to the ground now (`BaseTilt`).
    pub(in crate::view3d) tilt: Quat,
}

impl TokenView {
    /// How the base is turned to lie on the ground under it (none on flat
    /// ground): for things laid on its top, such as the target's reticule.
    pub fn tilt(&self) -> Quat {
        self.tilt
    }
}

/// Marks a token's base: the parent of its body, outline and their x-ray
/// copies, rotated by `place_camera_and_tokens` to lie on the ground.
#[derive(Component)]
pub struct TokenBase;

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
