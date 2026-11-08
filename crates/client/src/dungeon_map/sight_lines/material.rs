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
/// isn't a multiple of 16 bytes (the game quits on entering a site), so two
/// unused `f32`s pad the 24 bytes out to 32.
#[derive(ShaderType, Reflect, Clone, Copy, Debug, Default)]
pub struct MaskParams {
    /// The corner of the mask with the smallest world `x` and `z`.
    pub(super) origin: Vec2,
    /// One over the world size it covers.
    pub(super) per_unit: Vec2,
    side: f32,
    /// Seconds water's picture has flowed for (`mesh::flow`), wrapping; 0 on
    /// every material but the lit water's, and with Reduce Motion.
    flow_time: f32,
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
    /// Shows the whole surface (`None`, for the Hall of Worlds, where nothing
    /// blocks sight and the mask would cut off the far end), or only one side
    /// of the mask's edge again.
    pub fn set_side(&mut self, side: Option<MaskSide>) {
        let value = side.map_or(0.0, MaskSide::value);
        if self.params.side != value {
            self.params.side = value;
        }
    }

    /// How long water's picture has flowed, in seconds (0: still).
    pub fn flow_time(&self) -> f32 {
        self.params.flow_time
    }

    /// How long water's picture has flowed, in seconds.
    pub fn set_flow_time(&mut self, seconds: f32) {
        if self.params.flow_time != seconds {
            self.params.flow_time = seconds;
        }
    }

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
