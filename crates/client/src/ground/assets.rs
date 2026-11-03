//! What bags are drawn with: meshes, the weave picture, and sparkles' sizes.

use bevy::asset::RenderAssetUsages;
use bevy::image::{ImageAddressMode, ImageSampler, ImageSamplerDescriptor};
use bevy::mesh::{Indices, PrimitiveTopology};
use bevy::prelude::*;
use bevy::render::render_resource::{Extent3d, Face, TextureDimension, TextureFormat};
use shared::ground::Load;
use std::collections::BTreeMap;

use super::bag::{Model, size_of};
use super::{
    BagAssets, BagMeshes, LOWEST_SPARKLE, SPARKLE, SPARKLE_SCALE, STAR_EDGE, STAR_HOVER_OUTLINE,
    STAR_LONG, STAR_OUTLINE, STAR_SHORT, WEAVE_PIXELS,
};

/// How big a look's sparkle is drawn.
pub(super) fn sparkle_scale(load: Load) -> f32 {
    SPARKLE_SCALE * size_of(load).scale
}

/// How far up a look's sparkle stands.
pub(super) fn sparkle_up(load: Load) -> f32 {
    let size = size_of(load);
    (size.height * 0.85).max(LOWEST_SPARKLE * size.scale)
}

pub(super) fn to_mesh(model: Model) -> Mesh {
    Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::default(),
    )
    .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, model.positions)
    .with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, model.normals)
    .with_inserted_attribute(Mesh::ATTRIBUTE_UV_0, model.uvs)
    .with_inserted_attribute(Mesh::ATTRIBUTE_COLOR, model.colours)
    .with_inserted_indices(Indices::U32(model.indices))
}

/// Canvas: threads over and under, near enough white for its colour to come from the
/// mesh, drawn here so there is no file to ship for it.
pub(super) fn weave_picture() -> Image {
    let mut bytes = Vec::with_capacity(WEAVE_PIXELS * WEAVE_PIXELS * 4);
    for y in 0..WEAVE_PIXELS {
        for x in 0..WEAVE_PIXELS {
            let over = (x / 2 + y / 2).is_multiple_of(2);
            let edge = if over { x % 2 == 0 } else { y % 2 == 0 };
            let shade = match (over, edge) {
                (true, false) => 255,
                (false, false) => 228,
                (_, true) => 200,
            };
            bytes.extend([shade, shade, shade, 255]);
        }
    }
    let mut image = Image::new(
        Extent3d {
            width: WEAVE_PIXELS as u32,
            height: WEAVE_PIXELS as u32,
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        bytes,
        TextureFormat::Rgba8UnormSrgb,
        RenderAssetUsages::default(),
    );
    image.sampler = ImageSampler::Descriptor(ImageSamplerDescriptor {
        address_mode_u: ImageAddressMode::Repeat,
        address_mode_v: ImageAddressMode::Repeat,
        ..default()
    });
    image
}

pub(super) fn make_assets(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut images: ResMut<Assets<Image>>,
) {
    let mut bags = BTreeMap::new();
    for load in Load::ALL {
        let scale = sparkle_scale(load);
        for variety in 0..super::bag::VARIETIES {
            let model = super::bag::bag(load, variety);
            let star_outline = super::bag::star(STAR_LONG, STAR_SHORT, STAR_OUTLINE / scale);
            let star_hover_outline =
                super::bag::star(STAR_LONG, STAR_SHORT, STAR_HOVER_OUTLINE / scale);
            let remembered = meshes.add(to_mesh(model.greyed()));
            let outline = meshes.add(to_mesh(super::bag::outline(&model)));
            let hover_outline = meshes.add(to_mesh(super::bag::hover_outline(&model)));
            let lit = meshes.add(to_mesh(model));
            bags.insert(
                (load, variety),
                BagMeshes {
                    lit,
                    remembered,
                    outline,
                    hover_outline,
                    star_outline: meshes.add(to_mesh(star_outline)),
                    star_hover_outline: meshes.add(to_mesh(star_hover_outline)),
                },
            );
        }
    }
    let weave = images.add(weave_picture());
    commands.insert_resource(BagAssets {
        bags,
        star: meshes.add(to_mesh(super::bag::star(STAR_LONG, STAR_SHORT, 0.0))),
        lit: materials.add(StandardMaterial {
            base_color_texture: Some(weave.clone()),
            perceptual_roughness: 0.95,
            ..default()
        }),
        remembered: materials.add(StandardMaterial {
            base_color_texture: Some(weave),
            unlit: true,
            ..default()
        }),
        sparkle: materials.add(StandardMaterial {
            base_color: SPARKLE,
            unlit: true,
            ..default()
        }),
        // Drawn from inside, so only what stands out past the bag shows.
        outline: materials.add(StandardMaterial {
            base_color: Color::BLACK,
            unlit: true,
            cull_mode: Some(Face::Front),
            ..default()
        }),
        star_edge: materials.add(StandardMaterial {
            base_color: STAR_EDGE,
            unlit: true,
            cull_mode: None,
            ..default()
        }),
    });
}
