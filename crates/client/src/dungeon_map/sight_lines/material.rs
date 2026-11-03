//! Surfaces' material: the standard one, with the sight mask added.

use bevy::pbr::{ExtendedMaterial, MaterialExtension};
use bevy::prelude::*;
use bevy::render::render_resource::{AsBindGroup, ShaderType};
use bevy::shader::ShaderRef;

/// A surface's material: the standard one, shown only inside or only outside
/// what the mask says is in sight.
pub type SurfaceMaterial = ExtendedMaterial<StandardMaterial, SightMask>;

/// Which part of a surface to show. Values match `sight_mask.wesl`.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum MaskSide {
    InSight,
    OutOfSight,
}

impl MaskSide {
    fn value(self) -> f32 {
        match self {
            Self::InSight => 1.0,
            Self::OutOfSight => -1.0,
        }
    }
}

/// Order matches `struct Mask` in the shader. WebGL 2 refuses a uniform that
/// isn't a multiple of 16 bytes (the game quits on entering a site), so three
/// unused `f32`s pad the 20 bytes out to 32.
#[derive(ShaderType, Reflect, Clone, Copy, Debug, Default)]
pub struct MaskParams {
    /// The corner of the mask with the smallest world `x` and `z`.
    pub(super) origin: Vec2,
    /// One over the world size it covers.
    pub(super) per_unit: Vec2,
    side: f32,
    _pad_a: f32,
    _pad_b: f32,
    _pad_c: f32,
}

/// Adds the mask to a surface's standard material.
#[derive(Asset, AsBindGroup, Reflect, Debug, Clone)]
pub struct SightMask {
    #[uniform(100)]
    pub(super) params: MaskParams,
    #[texture(101)]
    #[sampler(102)]
    mask: Handle<Image>,
}

impl MaterialExtension for SightMask {
    fn fragment_shader() -> ShaderRef {
        "shaders/sight_mask.wesl".into()
    }
}

impl SightMask {
    pub fn new(mask: Handle<Image>, side: MaskSide) -> Self {
        Self {
            params: MaskParams {
                per_unit: Vec2::ONE,
                side: side.value(),
                ..default()
            },
            mask,
        }
    }
}
