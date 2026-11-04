//! The sky's materials: terrain with sky, clouds and precipitation.

use bevy::mesh::MeshVertexBufferLayoutRef;
use bevy::pbr::{ExtendedMaterial, MaterialExtension, MaterialPipeline, MaterialPipelineKey};
use bevy::prelude::*;
use bevy::render::render_resource::{
    AsBindGroup, RenderPipelineDescriptor, ShaderType, SpecializedMeshPipelineError,
};
use bevy::shader::ShaderRef;

pub type TerrainMaterial = ExtendedMaterial<StandardMaterial, TerrainSky>;

/// Everything the sky's shaders share. Order matches `struct Sky` in them.
#[derive(ShaderType, Reflect, Clone, Copy, Debug, Default)]
pub struct SkyParams {
    /// World position of each sky texture's corner, and their size.
    pub(super) origin_a: Vec2,
    pub(super) origin_b: Vec2,
    pub(super) extent: f32,
    /// How far from the first moment of the sky to the second, `0..=1`.
    pub(super) blend: f32,
    /// Seconds, for things moving.
    pub(super) time: f32,
    pub(super) cloud_height: f32,
    /// The terrain window (see `ViewSquare`).
    pub(super) half: f32,
    pub(super) right: Vec2,
    pub(super) towards: Vec2,
    pub(super) centre: Vec2,
    /// `rgb`: what to multiply colours by to look lit by the sky (`SkyNow::lit`).
    /// `a`: how strong the glow round the player is.
    pub(super) light: Vec4,
    /// The Video settings, each 0 (off) to 1: clouds shown, clouds shading the
    /// ground (needs shadows too), the share of rain drawn, foliage painted.
    pub(super) shown: Vec4,
    /// `x`: the least light on the land (Brightness); `y`: seconds for moving
    /// water, which Reduce Motion holds still.
    pub(super) still: Vec4,
}

/// Adds the sky to the terrain's standard material.
#[derive(Asset, AsBindGroup, Reflect, Debug, Clone)]
pub struct TerrainSky {
    #[uniform(100)]
    pub(super) params: SkyParams,
    #[texture(101)]
    #[sampler(102)]
    pub(super) sky_a: Handle<Image>,
    #[texture(103)]
    pub(super) sky_b: Handle<Image>,
}

impl MaterialExtension for TerrainSky {
    fn fragment_shader() -> ShaderRef {
        "shaders/terrain.wesl".into()
    }
}

/// The see-through sheet of cloud.
#[derive(Asset, TypePath, AsBindGroup, Debug, Clone)]
pub struct CloudMaterial {
    #[uniform(0)]
    pub(super) params: SkyParams,
    #[texture(1)]
    #[sampler(2)]
    pub(super) sky_a: Handle<Image>,
    #[texture(3)]
    pub(super) sky_b: Handle<Image>,
}

impl Material for CloudMaterial {
    fn fragment_shader() -> ShaderRef {
        "shaders/clouds.wesl".into()
    }

    fn alpha_mode(&self) -> AlphaMode {
        AlphaMode::Blend
    }

    fn enable_prepass() -> bool {
        false
    }

    fn enable_shadows() -> bool {
        false
    }
}

/// Falling rain and snow.
#[derive(Asset, TypePath, AsBindGroup, Debug, Clone)]
pub struct PrecipitationMaterial {
    #[uniform(0)]
    pub(super) params: SkyParams,
    #[texture(1)]
    #[sampler(2)]
    pub(super) sky_a: Handle<Image>,
    #[texture(3)]
    pub(super) sky_b: Handle<Image>,
}

impl Material for PrecipitationMaterial {
    fn vertex_shader() -> ShaderRef {
        "shaders/precipitation.wesl".into()
    }

    fn fragment_shader() -> ShaderRef {
        "shaders/precipitation.wesl".into()
    }

    fn alpha_mode(&self) -> AlphaMode {
        AlphaMode::Blend
    }

    fn enable_prepass() -> bool {
        false
    }

    fn enable_shadows() -> bool {
        false
    }

    fn specialize(
        _pipeline: &MaterialPipeline,
        descriptor: &mut RenderPipelineDescriptor,
        layout: &MeshVertexBufferLayoutRef,
        _key: MaterialPipelineKey<Self>,
    ) -> Result<(), SpecializedMeshPipelineError> {
        let vertex_layout = layout.0.get_layout(&[
            Mesh::ATTRIBUTE_POSITION.at_shader_location(0),
            Mesh::ATTRIBUTE_UV_0.at_shader_location(1),
        ])?;
        descriptor.vertex.buffers = vec![vertex_layout];
        Ok(())
    }
}
