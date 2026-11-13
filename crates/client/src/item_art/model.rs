//! Creatures as 3D models for the maps: the same builders as their pictures, in
//! millimetres at their real sizes, kept by look so a look is built once.

mod bones;

pub(super) use bones::{BodyPart, Bones};
mod mesh;
mod pieces;

use std::collections::HashMap;
use std::sync::Arc;

use bevy::prelude::*;

use super::Look;
use crate::common::{ModelMaterial, ModelMeshes, TokenModel, model_material};

/// Every creature model built so far, by look. Kept for good, as pictures are: the
/// same outfit is built once, and a changed one is a new look.
#[derive(Resource, Default)]
pub struct CreatureModels {
    built: HashMap<Look, TokenModel>,
}

impl CreatureModels {
    /// The model of a creature looking like this (`character_look`, `morph_look`),
    /// built now if it hasn't been; `None` for a look that isn't a body.
    pub fn model(
        &mut self,
        look: &Look,
        meshes: &mut Assets<Mesh>,
        materials: &mut Assets<ModelMaterial>,
    ) -> Option<TokenModel> {
        if let Some(model) = self.built.get(look) {
            return Some(model.clone());
        }
        let built = pieces::pieces(look)?;
        let bones = bones::Bones::of(&built.layout);
        let bounds = mesh::bounds_of(built.pieces.iter().flat_map(|(triangles, _)| triangles));
        let mut all = Vec::new();
        let mut parts = Vec::new();
        let (mut height, mut reach) = (0.0_f32, 0.0_f32);
        for (triangles, material) in built.pieces.into_iter() {
            for corner in triangles.iter().flatten() {
                height = height.max(corner.y);
                reach = reach.max(Vec2::new(corner.x, corner.z).length());
            }
            parts.push((
                meshes.add(mesh::part_mesh(&triangles, &bones, bounds)),
                materials.add(model_material(material)),
            ));
            all.extend(triangles);
        }
        let model = TokenModel(Arc::new(ModelMeshes {
            parts,
            hull: meshes.add(mesh::hull_mesh(&all, bounds)),
            skin: meshes.add(mesh::hull_mesh(&built.skin, bounds)),
            height,
            reach,
        }));
        self.built.insert(look.clone(), model.clone());
        Some(model)
    }

    /// How many looks have been built.
    #[cfg(test)]
    pub fn len(&self) -> usize {
        self.built.len()
    }
}

#[cfg(test)]
mod tests;
