//! X-ray: what is hidden behind something else, drawn through it in one
//! colour (`assets/shaders/xray.wesl`), so tokens and things the player should
//! know about (bags) show through walls and hills (owner, 2026-11-09).
//!
//! An x-ray part is a copy of a model's mesh with this material. Its pipeline
//! turns the depth test round: a pixel is drawn only where something nearer
//! has already been drawn there. Every part is in the transparent pass, after
//! everything solid, sorted back to front with `order` breaking ties within
//! one model:
//!
//! 1. the picture (`PICTURE_ORDER`), a faint silhouette;
//! 2. the body (`FILL_ORDER`), its front faces as a faint fill, writing its
//!    depth so that what comes next is not drawn over it;
//! 3. the outline (`OUTLINE_ORDER`), the black outline's mesh seen from the
//!    front: it shows only round the body's fill, as a coloured outline.
//!
//! Where a model is in plain sight, its own front faces are exactly as far as
//! what is drawn, so nothing of it passes (with a little depth bias to be
//! sure); the outline's front faces are nearer than the body.

use bevy::mesh::MeshVertexBufferLayoutRef;
use bevy::pbr::{MaterialPipeline, MaterialPipelineKey};
use bevy::prelude::*;
use bevy::render::render_resource::{
    AsBindGroup, CompareFunction, RenderPipelineDescriptor, SpecializedMeshPipelineError,
};
use bevy::shader::ShaderRef;

/// How much of the colour shows on a hidden body, and on a hidden picture.
const FILL_ALPHA: f32 = 0.25;
const PICTURE_ALPHA: f32 = 0.45;

/// Sort order within one model (`Material::depth_bias`: nearer the camera
/// draws later). The picture stands above its token's middle, so it sorts
/// nearer than its token's body: it is moved back past it.
const PICTURE_ORDER: f32 = -40.0;
const FILL_ORDER: f32 = 0.0;
const OUTLINE_ORDER: f32 = 0.01;

/// The colour x-ray outlines are for anything that isn't a token.
pub const XRAY_WHITE: Color = Color::WHITE;

/// Draws what is hidden behind something, in one colour.
#[derive(Asset, TypePath, AsBindGroup, Debug, Clone)]
#[bind_group_data(XrayKey)]
pub struct XrayMaterial {
    #[uniform(0)]
    colour: LinearRgba,
    /// A picture whose alpha cuts the shape out (an icon's), or none.
    #[texture(1)]
    #[sampler(2)]
    picture: Option<Handle<Image>>,
    order: f32,
    writes_depth: bool,
}

/// What the pipeline differs by.
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct XrayKey {
    writes_depth: bool,
}

impl From<&XrayMaterial> for XrayKey {
    fn from(material: &XrayMaterial) -> Self {
        Self {
            writes_depth: material.writes_depth,
        }
    }
}

impl XrayMaterial {
    /// A hidden model's outline, in `colour`: for its outline mesh.
    pub fn outline(colour: Color) -> Self {
        Self::new(colour, 1.0, None, OUTLINE_ORDER, false)
    }

    /// A hidden model's body, faintly in `colour`: for its own mesh.
    pub fn fill(colour: Color) -> Self {
        Self::new(colour, FILL_ALPHA, None, FILL_ORDER, true)
    }

    /// A hidden picture's silhouette in `colour`, cut out by `picture`.
    pub fn picture(colour: Color, picture: Handle<Image>) -> Self {
        Self::new(colour, PICTURE_ALPHA, Some(picture), PICTURE_ORDER, false)
    }

    fn new(
        colour: Color,
        alpha: f32,
        picture: Option<Handle<Image>>,
        order: f32,
        writes_depth: bool,
    ) -> Self {
        Self {
            colour: colour.with_alpha(alpha).to_linear(),
            picture,
            order,
            writes_depth,
        }
    }
}

impl Material for XrayMaterial {
    fn fragment_shader() -> ShaderRef {
        "shaders/xray.wesl".into()
    }

    fn alpha_mode(&self) -> AlphaMode {
        AlphaMode::Blend
    }

    fn depth_bias(&self) -> f32 {
        self.order
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
        _layout: &MeshVertexBufferLayoutRef,
        key: MaterialPipelineKey<Self>,
    ) -> Result<(), SpecializedMeshPipelineError> {
        if let Some(depth) = descriptor.depth_stencil.as_mut() {
            // Depth is reversed (1 is nearest): only what is further than what
            // is already drawn passes.
            depth.depth_compare = Some(CompareFunction::Less);
            depth.depth_write_enabled = Some(key.bind_group_data.writes_depth);
            // Nearer by a hair, so a model's faces in plain sight never pass.
            depth.bias.constant = 8;
            depth.bias.slope_scale = 1.0;
        }
        Ok(())
    }
}
