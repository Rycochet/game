//! Creatures' 3D models (`common::TokenModel`), standing on their tokens' bases in
//! place of their icons, with a black outline and their x-ray.

use bevy::light::NotShadowCaster;
use bevy::prelude::*;

use super::{HullMaterials, TokenView};
use crate::common::{ModelMaterial, Token, TokenModel};

/// World units a millimetre of a model is drawn: a person of the usual height stands
/// about as tall as the pictures did, on a base drawn to the same scale.
pub const MODEL_SCALE: f32 = 0.0124;

/// The point on top of a token's base a model stands on, turned about the
/// vertical the way its owner faces (`turn_models`), scaled from millimetres.
/// `shown` is the model and attitude colour it has; `angle` how far it is
/// turned now, in radians.
#[derive(Component, Default)]
pub struct ModelAnchor {
    pub(super) shown: Option<(TokenModel, usize)>,
    pub(super) angle: Option<f32>,
}

impl ModelAnchor {
    /// The model shown, if any.
    pub fn model(&self) -> Option<&TokenModel> {
        self.shown.as_ref().map(|(model, _)| model)
    }
}

/// One of a model's pieces, with the material it usually has (`flash` makes it red).
#[derive(Component)]
pub struct ModelPart(pub(super) Handle<ModelMaterial>);

/// Shows each token owner's `TokenModel` on its anchor, rebuilding it when the model or
/// how its owner feels changes, and taking it away when there is none.
pub fn sync_models(
    mut commands: Commands,
    hulls: Res<HullMaterials>,
    owners: Query<(&Token, Option<&TokenModel>)>,
    views: Query<&TokenView>,
    mut anchors: Query<(Entity, &mut ModelAnchor, &ChildOf)>,
    mut visibility: Query<&mut Visibility, Without<ModelAnchor>>,
) {
    for (anchor, mut shown, child_of) in &mut anchors {
        let Ok(view) = views.get(child_of.parent()) else {
            continue;
        };
        let wanted = owners
            .get(view.owner)
            .ok()
            .and_then(|(token, model)| model.map(|model| (model.clone(), token.attitude as usize)));
        if shown.shown == wanted {
            continue;
        }
        commands.entity(anchor).despawn_children();
        shown.shown = wanted.clone();
        // The model stands over its base and would count as hiding it, so the base's
        // x-ray only shows without one; the model's own shows where it is.
        let no_base = owners.get(view.owner).is_ok_and(|(token, _)| token.no_base);
        let base_xray = match no_base || wanted.is_some() {
            true => Visibility::Hidden,
            false => Visibility::Inherited,
        };
        for part in [view.xray_body, view.xray_outline] {
            if let Ok(mut shown) = visibility.get_mut(part) {
                shown.set_if_neq(base_xray);
            }
        }
        let Some((model, attitude)) = wanted else {
            continue;
        };
        let meshes = &model.0;
        commands.entity(anchor).with_children(|parent| {
            for (mesh, material) in &meshes.parts {
                parent.spawn((
                    Mesh3d(mesh.clone()),
                    MeshMaterial3d(material.clone()),
                    ModelPart(material.clone()),
                    NotShadowCaster,
                ));
            }
            parent.spawn((
                Mesh3d(meshes.hull.clone()),
                MeshMaterial3d(hulls.black.clone()),
                NotShadowCaster,
            ));
            // Through whatever hides it: a faint fill and the outline, in its colour.
            parent.spawn((
                Mesh3d(meshes.skin.clone()),
                MeshMaterial3d(hulls.fill[attitude].clone()),
                NotShadowCaster,
            ));
            parent.spawn((
                Mesh3d(meshes.skin.clone()),
                MeshMaterial3d(hulls.outline[attitude].clone()),
                NotShadowCaster,
            ));
        });
    }
}
