//! Models' outlines and x-ray: a model's smooth hull, grown in the shader
//! (`assets/shaders/hull.wesl`) by `OUTLINE_PIXELS` on screen and pushed back from
//! the camera, black, seen from inside; and through whatever hides it (with the
//! x-ray's pipeline, `xray`) a faint fill and an outline in an attitude's colour of
//! its skin, drawn before the model itself, so only walls, hills and the like (and
//! creatures further back) count as hiding it, never its own clothes or limbs.

use bevy::mesh::MeshVertexBufferLayoutRef;
use bevy::pbr::{MaterialPipeline, MaterialPipelineKey};
use bevy::prelude::*;
use bevy::render::render_resource::{
    AsBindGroup, CompareFunction, Face, RenderPipelineDescriptor, ShaderType,
    SpecializedMeshPipelineError,
};
use bevy::shader::ShaderRef;
use bevy::window::PrimaryWindow;

use super::super::camera::OrbitCamera;
use super::flash::FLASH_RED;
use super::xray::XRAY_FIRST;
use super::{MODEL_SCALE, outline_width};
use crate::common::{Attitude, ModelMaterial, model_material};

/// How far the black outline is pushed back from the camera, as a share of how far it
/// is grown: lines show only where one part stands this far clear of what is behind.
const PUSH_SHARE: f32 = 1.5;

/// How much wider than the black outline a glow is grown.
const GLOW_SHARE: f32 = 3.0;

/// How far the x-ray is pulled towards the camera, in millimetres of the model, so its
/// soles never count as hidden by the base it stands on.
const PULL_MM: f32 = 40.0;

/// Sort order (`Material::depth_bias`, world units: more is drawn later): a model's
/// x-ray, fill then outline, with every other x-ray (`XRAY_FIRST`), before any model's
/// pieces and black outline (0), which then cover it wherever it is in plain sight.
const FILL_ORDER: f32 = XRAY_FIRST;
const OUTLINE_ORDER: f32 = XRAY_FIRST + 0.01;

/// How much of the colour shows on a hidden model (as `xray`'s fill).
const FILL_ALPHA: f32 = 0.25;

/// Order matches `struct Hull` in the shader.
#[derive(ShaderType, Clone, Copy, Debug, Default)]
struct HullParams {
    colour: Vec4,
    grow: f32,
    push: f32,
    unused_a: f32,
    unused_b: f32,
}

/// What a hull is drawn as.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum HullKind {
    /// The black outline.
    Black,
    /// Through what hides it: the faint fill, writing its depth so the outline only
    /// shows round it, then the outline (`xray`'s order).
    XrayFill,
    XrayOutline,
    /// A glow round the model (`common::TokenGlow`): wider than the black
    /// outline and behind it, so it shows as a band of colour beyond it.
    Glow,
}

/// A model's hull, drawn as its `HullKind`.
#[derive(Asset, TypePath, AsBindGroup, Debug, Clone)]
#[bind_group_data(HullKey)]
pub struct HullMaterial {
    #[uniform(0)]
    params: HullParams,
    kind: HullKind,
}

/// What the pipeline differs by.
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct HullKey {
    kind: HullKind,
}

impl From<&HullMaterial> for HullKey {
    fn from(material: &HullMaterial) -> Self {
        Self {
            kind: material.kind,
        }
    }
}

impl HullMaterial {
    fn new(colour: Color, kind: HullKind) -> Self {
        Self {
            params: HullParams {
                colour: colour.to_linear().to_vec4(),
                ..default()
            },
            kind,
        }
    }

    /// Its growth and push for an outline `outline` world units wide.
    fn follow(&mut self, outline: f32) {
        let pull = -PULL_MM * MODEL_SCALE;
        (self.params.grow, self.params.push) = match self.kind {
            HullKind::Black => (outline, outline * PUSH_SHARE),
            HullKind::XrayFill => (0.0, pull),
            HullKind::XrayOutline => (outline, pull),
            HullKind::Glow => (outline * GLOW_SHARE, outline * GLOW_SHARE * PUSH_SHARE),
        };
    }
}

impl Material for HullMaterial {
    fn vertex_shader() -> ShaderRef {
        "shaders/hull.wesl".into()
    }

    fn fragment_shader() -> ShaderRef {
        "shaders/hull.wesl".into()
    }

    fn alpha_mode(&self) -> AlphaMode {
        // All with the see-through things, the black one too, so that, as its model,
        // it covers the x-ray of what is behind it (`common::ModelMaterial`).
        AlphaMode::Blend
    }

    fn depth_bias(&self) -> f32 {
        match self.kind {
            HullKind::Black | HullKind::Glow => 0.0,
            HullKind::XrayFill => FILL_ORDER,
            HullKind::XrayOutline => OUTLINE_ORDER,
        }
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
        key: MaterialPipelineKey<Self>,
    ) -> Result<(), SpecializedMeshPipelineError> {
        let vertex_layout = layout.0.get_layout(&[
            Mesh::ATTRIBUTE_POSITION.at_shader_location(0),
            Mesh::ATTRIBUTE_NORMAL.at_shader_location(1),
        ])?;
        descriptor.vertex.buffers = vec![vertex_layout];
        let kind = key.bind_group_data.kind;
        // Black, seen from inside: only what stands out round the model shows. The
        // x-ray, seen from outside, as a token's.
        descriptor.primitive.cull_mode = Some(match kind {
            HullKind::Black | HullKind::Glow => Face::Front,
            _ => Face::Back,
        });
        if matches!(kind, HullKind::Black | HullKind::Glow) {
            // Solid after all, writing its depth, as `common::InFront`.
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
        } else if let Some(depth) = descriptor.depth_stencil.as_mut() {
            // As `XrayMaterial`: only what is further than what is drawn passes.
            depth.depth_compare = Some(CompareFunction::Less);
            depth.depth_write_enabled = Some(kind == HullKind::XrayFill);
            depth.bias.constant = 8;
            depth.bias.slope_scale = 1.0;
        }
        Ok(())
    }
}

/// The hulls' materials every model shares: black, and through walls in each
/// attitude's colour. Their growth follows the window and the zoom (`grow_hulls`).
#[derive(Resource)]
pub struct HullMaterials {
    pub(super) black: Handle<HullMaterial>,
    /// What a model's pieces are drawn with while it flashes (`flash`).
    pub(super) flash: Handle<ModelMaterial>,
    pub(super) fill: Vec<Handle<HullMaterial>>,
    pub(super) outline: Vec<Handle<HullMaterial>>,
    /// Glows by colour, made as they are first wanted.
    glows: Vec<([u8; 3], Handle<HullMaterial>)>,
    grown: (f32, f32),
}

impl HullMaterials {
    /// The glow in this colour, made the first time it is wanted.
    pub(super) fn glow(
        &mut self,
        rgb: [u8; 3],
        materials: &mut Assets<HullMaterial>,
    ) -> Handle<HullMaterial> {
        if let Some((_, handle)) = self.glows.iter().find(|(colour, _)| *colour == rgb) {
            return handle.clone();
        }
        let colour = Color::srgb_u8(rgb[0], rgb[1], rgb[2]);
        let mut material = HullMaterial::new(colour, HullKind::Glow);
        material.follow(outline_now(self.grown));
        let handle = materials.add(material);
        self.glows.push((rgb, handle.clone()));
        handle
    }
}

/// The outline's width for what the hulls were last grown for.
fn outline_now((height, distance): (f32, f32)) -> f32 {
    if height == 0.0 {
        0.0
    } else {
        outline_width(height, distance)
    }
}

impl FromWorld for HullMaterials {
    fn from_world(world: &mut World) -> Self {
        let mut materials = world.resource_mut::<Assets<HullMaterial>>();
        let black = materials.add(HullMaterial::new(Color::BLACK, HullKind::Black));
        let mut each = |kind: HullKind, alpha: f32| -> Vec<Handle<HullMaterial>> {
            Attitude::ALL
                .iter()
                .map(|attitude| {
                    let colour = attitude.colour().with_alpha(alpha);
                    materials.add(HullMaterial::new(colour, kind))
                })
                .collect()
        };
        let fill = each(HullKind::XrayFill, FILL_ALPHA);
        let outline = each(HullKind::XrayOutline, 1.0);
        let flash = world
            .resource_mut::<Assets<ModelMaterial>>()
            .add(model_material(StandardMaterial {
                base_color: FLASH_RED,
                unlit: true,
                ..default()
            }));
        Self {
            black,
            flash,
            fill,
            outline,
            glows: Vec::new(),
            grown: (0.0, 0.0),
        }
    }
}

/// Keeps outlines `OUTLINE_PIXELS` wide as the window's height or the zoom changes.
pub fn grow_hulls(
    window: Single<&Window, With<PrimaryWindow>>,
    camera: Single<&OrbitCamera>,
    mut hulls: ResMut<HullMaterials>,
    mut materials: ResMut<Assets<HullMaterial>>,
) {
    let height = window.physical_height() as f32;
    let now = (height, camera.distance);
    // Minimised windows have no height; keep the old sizes.
    if height == 0.0 || hulls.grown == now {
        return;
    }
    hulls.grown = now;
    let outline = outline_width(height, camera.distance);
    let glows = hulls.glows.iter().map(|(_, handle)| handle);
    let all = std::iter::once(&hulls.black)
        .chain(&hulls.fill)
        .chain(&hulls.outline)
        .chain(glows);
    for handle in all {
        if let Some(mut material) = materials.get_mut(handle) {
            material.follow(outline);
        }
    }
}
