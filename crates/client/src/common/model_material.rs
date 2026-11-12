//! What creatures' models are drawn with: the usual lit material, drawn with the
//! see-through things (after every x-ray behind them) while still writing its depth.

use bevy::mesh::MeshVertexBufferLayoutRef;
use bevy::pbr::{
    ExtendedMaterial, MaterialExtension, MaterialExtensionKey, MaterialExtensionPipeline,
};
use bevy::prelude::*;
use bevy::render::render_resource::{
    AsBindGroup, RenderPipelineDescriptor, SpecializedMeshPipelineError,
};

/// A piece of a creature's model (`TokenModel`). Its base must be `AlphaMode::Blend`
/// (`model_material`) so it is drawn after everything solid, sorted with the
/// see-through things: the x-ray of what stands behind it (a bag, stairs, another
/// creature) is then drawn first and covered, instead of painting over it.
pub type ModelMaterial = ExtendedMaterial<StandardMaterial, InFront>;

/// Turns a blended material back into a solid one that writes its depth, as it is
/// whole (alpha 1): pieces of one model, sorted together, still hide each other right.
#[derive(Asset, AsBindGroup, TypePath, Debug, Clone, Default)]
pub struct InFront {}

impl MaterialExtension for InFront {
    fn enable_prepass() -> bool {
        false
    }

    fn enable_shadows() -> bool {
        false
    }

    fn specialize(
        _pipeline: &MaterialExtensionPipeline,
        descriptor: &mut RenderPipelineDescriptor,
        _layout: &MeshVertexBufferLayoutRef,
        _key: MaterialExtensionKey<Self>,
    ) -> Result<(), SpecializedMeshPipelineError> {
        if let Some(depth) = descriptor.depth_stencil.as_mut() {
            depth.depth_write_enabled = Some(true);
        }
        if let Some(target) = descriptor
            .fragment
            .as_mut()
            .and_then(|fragment| fragment.targets.first_mut())
            .and_then(Option::as_mut)
        {
            target.blend = None;
        }
        Ok(())
    }
}

/// A model's piece made of this.
pub fn model_material(base: StandardMaterial) -> ModelMaterial {
    ModelMaterial {
        base: StandardMaterial {
            alpha_mode: AlphaMode::Blend,
            ..base
        },
        extension: InFront {},
    }
}
