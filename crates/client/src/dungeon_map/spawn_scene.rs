//! Making the scene: its materials, meshes, lights and mask.

use bevy::camera::visibility::NoFrustumCulling;
use bevy::image::{ImageLoaderSettings, ImageSampler};
use bevy::light::NotShadowCaster;
use bevy::prelude::*;
use std::collections::BTreeMap;

use super::mesh::Surface;
use super::sight_lines::{MaskSide, SightMask, SurfaceMaterial};
use super::theme::Theme;
use super::{
    BLEND_DEPTH_BIAS, CropPart, DungeonScene, HerbPart, MEMORY_ALPHA, MemoryPart,
    SEE_THROUGH_ALPHA, SeeThroughPart, SolidPart, StairsThroughPart, SurfaceMaterials,
};
use crate::common::GameView;

#[allow(clippy::too_many_arguments)]
pub(super) fn spawn_scene(
    mut commands: Commands,
    view: Res<State<GameView>>,
    asset_server: Res<AssetServer>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut surface_materials: ResMut<Assets<SurfaceMaterial>>,
    mut images: ResMut<Assets<Image>>,
    mut rain_materials: ResMut<Assets<super::rain::RainMaterial>>,
) {
    let mask =
        super::sight_lines::spawn_mask(&mut commands, &mut images, &mut meshes, &mut materials);
    // Everything is cut at the edge of sight: what is lit shows inside it, what
    // is remembered outside.
    let masked = |base: StandardMaterial, side: MaskSide| SurfaceMaterial {
        base,
        extension: SightMask::new(mask.clone(), side),
    };
    let theme = Theme::of(None);
    let pictures: Vec<Handle<Image>> = Surface::ALL
        .iter()
        .map(|&surface| load_tile(&asset_server, theme.picture(surface)))
        .collect();
    // Unlit: the light is in the vertex colours, worked out on the grid.
    // Memory shows the coloured pictures until their grey copies are made.
    let unlit = |picture: &Handle<Image>| StandardMaterial {
        base_color_texture: Some(picture.clone()),
        unlit: true,
        ..default()
    };
    let solid: Vec<_> = Surface::ALL
        .iter()
        .zip(&pictures)
        .map(|(&surface, picture)| {
            let opacity = super::liquid::opacity(surface);
            let base = if surface == Surface::WallBlend {
                StandardMaterial {
                    alpha_mode: AlphaMode::Blend,
                    depth_bias: BLEND_DEPTH_BIAS,
                    ..unlit(picture)
                }
            } else if opacity < 1.0 {
                // A surface with the bed showing through it, drawn before
                // anything blended that stands in it.
                StandardMaterial {
                    base_color: Color::srgba(1.0, 1.0, 1.0, opacity),
                    alpha_mode: AlphaMode::Blend,
                    depth_bias: -1.0,
                    ..unlit(picture)
                }
            } else {
                unlit(picture)
            };
            surface_materials.add(masked(base, MaskSide::InSight))
        })
        .collect();
    // Blended meshes don't sort against each other by themselves: floors are
    // pushed back so they are always drawn before the walls standing on them,
    // with the wall's fade across the floor in between.
    let memory: Vec<_> = Surface::ALL
        .iter()
        .zip(&pictures)
        .map(|(&surface, picture)| {
            let base = StandardMaterial {
                base_color: Color::srgba(1.0, 1.0, 1.0, MEMORY_ALPHA),
                // Solid even as memory: a whole town is remembered at once,
                // and blended meshes wouldn't sort.
                alpha_mode: if surface == Surface::Town {
                    AlphaMode::Opaque
                } else {
                    AlphaMode::Blend
                },
                depth_bias: match surface {
                    Surface::Wall | Surface::Town => 0.0,
                    Surface::WallBlend => BLEND_DEPTH_BIAS,
                    _ => -1.0,
                },
                ..unlit(picture)
            };
            surface_materials.add(masked(base, MaskSide::OutOfSight))
        })
        .collect();
    let see_through = surface_materials.add(masked(
        StandardMaterial {
            base_color: Color::srgba(1.0, 1.0, 1.0, SEE_THROUGH_ALPHA),
            base_color_texture: Some(load_tile(&asset_server, theme.wall)),
            unlit: true,
            alpha_mode: AlphaMode::Blend,
            // These sides face away from the camera, so they are back faces.
            cull_mode: None,
            ..default()
        },
        MaskSide::InSight,
    ));
    // Back faces stay culled, so a model doesn't show its far side as well.
    let stairs_through: Vec<_> = pictures
        .iter()
        .map(|picture| {
            surface_materials.add(masked(
                StandardMaterial {
                    base_color: Color::srgba(1.0, 1.0, 1.0, SEE_THROUGH_ALPHA),
                    alpha_mode: AlphaMode::Blend,
                    ..unlit(picture)
                },
                MaskSide::InSight,
            ))
        })
        .collect();
    let visibility = if *view.get() == GameView::Dungeon {
        Visibility::Inherited
    } else {
        Visibility::Hidden
    };
    let scene = commands
        .spawn((DungeonScene, Transform::default(), visibility))
        .id();
    for (&surface, material) in Surface::ALL.iter().zip(&solid) {
        commands.entity(scene).with_child((
            SolidPart(surface),
            // Placeholder until the first update builds the real mesh.
            Mesh3d(meshes.add(Mesh::from(Cuboid::from_length(0.0)))),
            MeshMaterial3d(material.clone()),
            // The mesh is replaced as things change, so its bounds go stale.
            NoFrustumCulling,
            NotShadowCaster,
        ));
    }
    for (&surface, material) in Surface::ALL.iter().zip(&memory) {
        commands.entity(scene).with_child((
            MemoryPart(surface),
            Mesh3d(meshes.add(Mesh::from(Cuboid::from_length(0.0)))),
            MeshMaterial3d(material.clone()),
            NoFrustumCulling,
            NotShadowCaster,
        ));
    }
    commands.entity(scene).with_child((
        SeeThroughPart,
        Mesh3d(meshes.add(Mesh::from(Cuboid::from_length(0.0)))),
        MeshMaterial3d(see_through.clone()),
        NoFrustumCulling,
        NotShadowCaster,
    ));
    for (&surface, material) in Surface::ALL.iter().zip(&stairs_through) {
        commands.entity(scene).with_child((
            StairsThroughPart(surface),
            Mesh3d(meshes.add(Mesh::from(Cuboid::from_length(0.0)))),
            MeshMaterial3d(material.clone()),
            NoFrustumCulling,
            NotShadowCaster,
        ));
    }
    commands.entity(scene).with_child((
        CropPart,
        Mesh3d(meshes.add(Mesh::from(Cuboid::from_length(0.0)))),
        MeshMaterial3d(surface_materials.add(masked(
            StandardMaterial {
                base_color_texture: Some(load_tile(&asset_server, super::field::CROPS_PICTURE)),
                unlit: true,
                alpha_mode: AlphaMode::Mask(0.5),
                // Seen from both sides.
                cull_mode: None,
                ..default()
            },
            MaskSide::InSight,
        ))),
        NoFrustumCulling,
        NotShadowCaster,
    ));
    commands.entity(scene).with_child((
        HerbPart,
        Mesh3d(meshes.add(Mesh::from(Cuboid::from_length(0.0)))),
        MeshMaterial3d(surface_materials.add(masked(
            StandardMaterial {
                base_color_texture: Some(load_tile(&asset_server, super::herbs::HERBS_PICTURE)),
                unlit: true,
                alpha_mode: AlphaMode::Mask(0.5),
                // Seen from both sides.
                cull_mode: None,
                ..default()
            },
            MaskSide::InSight,
        ))),
        NoFrustumCulling,
        NotShadowCaster,
    ));
    super::rain::spawn(&mut commands, scene, &mut meshes, &mut rain_materials);
    commands.insert_resource(SurfaceMaterials {
        solid,
        memory,
        see_through,
        stairs_through,
        ground_picture: pictures[Surface::Ground as usize].clone(),
        to_grey: pictures.into_iter().enumerate().collect(),
        greys: BTreeMap::new(),
    });
}

/// Loads a tile picture, sampled pixel by pixel to keep the pixel art crisp.
pub(super) fn load_tile(asset_server: &AssetServer, path: &'static str) -> Handle<Image> {
    asset_server
        .load_builder()
        .with_settings(|settings: &mut ImageLoaderSettings| {
            settings.sampler = ImageSampler::nearest();
        })
        .load(path)
}
