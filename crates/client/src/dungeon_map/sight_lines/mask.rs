//! The sight mask: what it is worked out from, and making it.

use bevy::camera::visibility::{NoFrustumCulling, RenderLayers};
use bevy::camera::{RenderTarget, ScalingMode};
use bevy::core_pipeline::tonemapping::Tonemapping;
use bevy::prelude::*;
use bevy::render::render_resource::TextureFormat;
use shared::dungeon::Sight;

use super::{Area, MASK_LAYER, MASK_PIXELS, OutlineMesh, Vec2d, WallMap};

/// What the outlines are worked out from, with what was last worked out.
#[derive(Resource)]
pub struct SightLines {
    pub(super) walls: WallMap,
    /// Where each viewer was when last worked out.
    pub(super) from: Vec<Vec2d>,
    /// Open cells any outline reaches into, and the walls touching them.
    pub touched: Option<Sight>,
    /// The same for each viewer: the player first, then those sharing sight.
    pub touched_by: Vec<Sight>,
    /// Cells an outline's edge crosses, or close to it: only part of each is
    /// in sight.
    pub crossed: Option<Sight>,
    /// Counts the times `touched` changed.
    pub changes: u64,
    /// How long the outlines last took, for `ViewStats`.
    pub cost_ms: f32,
    pub(super) outline_mesh: Handle<Mesh>,
    pub(super) area: Option<Area>,
}

/// Marks the mask's camera.
#[derive(Component)]
pub struct MaskCamera;

/// Makes the mask with nothing in sight yet, and what draws to it.
pub fn spawn_mask(
    commands: &mut Commands,
    images: &mut Assets<Image>,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
) -> Handle<Image> {
    let mask = images.add(Image::new_target_texture(
        MASK_PIXELS,
        MASK_PIXELS,
        TextureFormat::Rgba8Unorm,
        None,
    ));
    let outline_mesh = meshes.add(Mesh::from(Cuboid::from_length(0.0)));
    commands.spawn((
        MaskCamera,
        Camera3d::default(),
        Camera {
            // Before the view's camera, which shows it.
            order: -1,
            clear_color: Color::BLACK.into(),
            is_active: false,
            ..default()
        },
        RenderTarget::Image(mask.clone().into()),
        Projection::Orthographic(OrthographicProjection {
            near: 0.0,
            far: 2.0,
            scaling_mode: ScalingMode::Fixed {
                width: 1.0,
                height: 1.0,
            },
            ..OrthographicProjection::default_3d()
        }),
        Tonemapping::None,
        Msaa::Off,
        Transform::from_xyz(0.0, 1.0, 0.0).looking_to(Vec3::NEG_Y, Vec3::NEG_Z),
        RenderLayers::layer(MASK_LAYER),
    ));
    commands.spawn((
        Mesh3d(outline_mesh.clone()),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: Color::WHITE,
            unlit: true,
            cull_mode: None,
            ..default()
        })),
        Transform::default(),
        Visibility::Hidden,
        NoFrustumCulling,
        OutlineMesh,
        RenderLayers::layer(MASK_LAYER),
    ));
    commands.insert_resource(SightLines {
        walls: WallMap::default(),
        from: Vec::new(),
        touched: None,
        touched_by: Vec::new(),
        crossed: None,
        changes: 0,
        cost_ms: 0.0,
        outline_mesh,
        area: None,
    });
    mask
}
