//! A creature's 3D model, shown standing on its token in place of its icon.

use std::sync::Arc;

use bevy::prelude::*;

/// A 3D model shown standing on an entity's token in place of its `Icon` (a creature,
/// as it is dressed). Made by `item_art`, kept by its look, so creatures that look the
/// same share one; `view3d` draws it. Cheap to clone.
#[derive(Component, Clone, Debug)]
pub struct TokenModel(pub Arc<ModelMeshes>);

impl PartialEq for TokenModel {
    /// The same model, not merely one that looks alike.
    fn eq(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.0, &other.0)
    }
}

/// The meshes of a model, in millimetres, standing on the ground at the origin with
/// its front towards +z (its left towards +x), in its rest pose. Each vertex of
/// `parts` carries the body part it moves with (`item_art`'s `Vertex_BodyPart`),
/// for animation later.
#[derive(Debug)]
pub struct ModelMeshes {
    /// Each piece in what it is made of.
    pub parts: Vec<(Handle<Mesh>, Handle<super::ModelMaterial>)>,
    /// All of it as one surface with smooth normals, to grow for the outline.
    pub hull: Handle<Mesh>,
    /// The body's own skin alone, the same way, for the x-ray: its outline round the
    /// body, not round every piece worn.
    pub skin: Handle<Mesh>,
    /// How tall it stands and how far it reaches from its middle, in millimetres.
    pub height: f32,
    pub reach: f32,
}
