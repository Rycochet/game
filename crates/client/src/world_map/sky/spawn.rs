//! Making the sky's assets: textures, the cloud sheet and the drops.

use bevy::asset::RenderAssetUsages;
use bevy::camera::visibility::NoFrustumCulling;
use bevy::image::ImageSampler;
use bevy::light::NotShadowCaster;
use bevy::mesh::{Indices, PrimitiveTopology};
use bevy::prelude::*;
use bevy::render::render_resource::{Extent3d, TextureDimension, TextureFormat};
use shared::rng::Seed;

use super::super::{FollowsPlayer, WorldMapScene};
use super::{
    CloudMaterial, DROPS, PrecipitationMaterial, SKY_SIDE, SkyParams, TerrainMaterial, TerrainSky,
};

/// The sky textures and what shows them.
#[derive(Resource)]
pub struct SkyAssets {
    /// The older moment, then the newer. Swapped as time goes on.
    pub(super) moments: [Handle<Image>; 2],
    pub terrain: Handle<TerrainMaterial>,
    /// The same, for rivers: drawn from both sides.
    pub rivers: Handle<TerrainMaterial>,
    pub(super) clouds: Handle<CloudMaterial>,
    pub(super) precipitation: Handle<PrecipitationMaterial>,
    pub(super) cloud_sheet: Handle<Mesh>,
}

/// Makes the sky's assets and spawns the cloud sheet and precipitation.
#[allow(clippy::too_many_arguments)]
pub fn spawn(
    commands: &mut Commands,
    visibility: Visibility,
    images: &mut Assets<Image>,
    meshes: &mut Assets<Mesh>,
    terrain_materials: &mut Assets<TerrainMaterial>,
    clouds: &mut Assets<CloudMaterial>,
    precipitation: &mut Assets<PrecipitationMaterial>,
) {
    let mut new_moment = || {
        let mut image = Image::new_fill(
            Extent3d {
                width: SKY_SIDE as u32,
                height: SKY_SIDE as u32,
                depth_or_array_layers: 1,
            },
            TextureDimension::D2,
            &[0, 0, 128, 255],
            TextureFormat::Rgba8Unorm,
            RenderAssetUsages::default(),
        );
        image.sampler = ImageSampler::linear();
        images.add(image)
    };
    let moments = [new_moment(), new_moment()];

    let params = SkyParams::default();
    let terrain_material = |cull_mode| TerrainMaterial {
        base: StandardMaterial {
            base_color: Color::WHITE,
            perceptual_roughness: 1.0,
            cull_mode,
            ..default()
        },
        extension: TerrainSky {
            params,
            sky_a: moments[0].clone(),
            sky_b: moments[1].clone(),
        },
    };
    let terrain = terrain_materials.add(terrain_material(StandardMaterial::default().cull_mode));
    // River strips may turn either way as they wander.
    let rivers = terrain_materials.add(terrain_material(None));

    let cloud_material = clouds.add(CloudMaterial {
        params,
        sky_a: moments[0].clone(),
        sky_b: moments[1].clone(),
    });
    let cloud_sheet = meshes.add(Mesh::from(Cuboid::from_length(0.0)));
    commands.spawn((
        WorldMapScene,
        FollowsPlayer,
        Mesh3d(cloud_sheet.clone()),
        MeshMaterial3d(cloud_material.clone()),
        NoFrustumCulling,
        NotShadowCaster,
        Transform::default(),
        visibility,
    ));

    let drops = precipitation.add(PrecipitationMaterial {
        params,
        sky_a: moments[0].clone(),
        sky_b: moments[1].clone(),
    });
    commands.spawn((
        WorldMapScene,
        Mesh3d(meshes.add(drops_mesh())),
        MeshMaterial3d(drops.clone()),
        NoFrustumCulling,
        NotShadowCaster,
        Transform::default(),
        visibility,
    ));

    commands.insert_resource(SkyAssets {
        moments,
        terrain,
        rivers,
        clouds: cloud_material,
        precipitation: drops,
        cloud_sheet,
    });
}

/// `DROPS` quads whose positions are random numbers, for the shader to place.
pub(super) fn drops_mesh() -> Mesh {
    let mut rng = Seed::new(1).rng();
    let mut positions = Vec::new();
    let mut corners = Vec::new();
    let mut indices = Vec::new();
    for drop in 0..DROPS {
        let seed = [
            rng.range_f64(0.0, 1.0) as f32,
            rng.range_f64(0.0, 1.0) as f32,
            rng.range_f64(0.0, 1.0) as f32,
        ];
        for corner in [[-0.5f32, 0.5], [-0.5, -0.5], [0.5, -0.5], [0.5, 0.5]] {
            positions.push(seed);
            corners.push(corner);
        }
        let first = drop * 4;
        indices.extend_from_slice(&[first, first + 1, first + 2, first, first + 2, first + 3]);
    }
    Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::default(),
    )
    .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, positions)
    .with_inserted_attribute(Mesh::ATTRIBUTE_UV_0, corners)
    .with_inserted_indices(Indices::U32(indices))
}
