//! The black edge round every picture (`assets/shaders/picture_edge.wesl`): drawn
//! flat over a page of pictures, it makes every pixel near enough to one of a
//! model black, and puts the model over it.

use bevy::prelude::*;
use bevy::render::render_resource::{AsBindGroup, ShaderType};
use bevy::shader::ShaderRef;

/// Order matches `struct Edge` in the shader.
#[derive(ShaderType, Clone, Copy, Debug)]
struct EdgeParams {
    /// How far the edge reaches, in the page's pixels.
    radius: f32,
}

/// Shows a page of models with their edges, as thick as `radius`.
#[derive(Asset, TypePath, AsBindGroup, Debug, Clone)]
pub(super) struct EdgeMaterial {
    #[uniform(0)]
    params: EdgeParams,
    #[texture(1)]
    #[sampler(2)]
    models: Handle<Image>,
}

impl EdgeMaterial {
    pub(super) fn new(radius: f32, models: Handle<Image>) -> Self {
        Self {
            params: EdgeParams { radius },
            models,
        }
    }
}

impl Material for EdgeMaterial {
    fn fragment_shader() -> ShaderRef {
        "shaders/picture_edge.wesl".into()
    }
}
