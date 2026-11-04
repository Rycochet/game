//! The sheet of something like running water filling a gate's opening: a flat
//! face over the opening's shape, its picture tiling once per `TILE_CELLS`.

use bevy::asset::RenderAssetUsages;
use bevy::image::{ImageAddressMode, ImageSampler, ImageSamplerDescriptor};
use bevy::mesh::{Indices, PrimitiveTopology};
use bevy::prelude::*;
use bevy::render::render_resource::{Extent3d, TextureDimension, TextureFormat};
use shared::hall::GateDesign;

use super::profile::{SPRING, jamb, top};

/// How many cells one copy of the picture covers, across and down.
const TILE_CELLS: f32 = 0.5;
/// The picture's size: narrow and tall, so its streaks run down.
const WIDTH: u32 = 32;
const HEIGHT: u32 = 64;
/// The water's blue, its streaks' paler blue, and how see-through each is.
const BLUE: [u8; 3] = [42, 112, 204];
const STREAK: [u8; 3] = [178, 220, 255];
const BLUE_ALPHA: f32 = 0.45;
const STREAK_ALPHA: f32 = 0.8;

/// The opening's shape as a mesh, `scale` the size of a cell, facing both
/// ways (draw it without culling).
pub(super) fn mesh(design: GateDesign, scale: f32) -> Mesh {
    let jamb = jamb(design);
    let mut ring = vec![Vec2::new(-jamb, 0.0)];
    ring.extend(top(design).into_iter().map(|p| p + Vec2::new(0.0, SPRING)));
    ring.push(Vec2::new(jamb, 0.0));
    // A fan from the jambs' tops, which sees all of the opening (as the frame
    // is filled), closed along the ground.
    let middle = Vec2::new(0.0, SPRING);
    let mut points = vec![middle];
    points.extend(ring.iter().copied());
    let mut indices = Vec::new();
    for i in 1..points.len() as u32 {
        let next = if i + 1 == points.len() as u32 {
            1
        } else {
            i + 1
        };
        indices.extend([0, i, next]);
    }
    let positions: Vec<Vec3> = points.iter().map(|p| p.extend(0.0) * scale).collect();
    let uvs: Vec<[f32; 2]> = points
        .iter()
        .map(|p| [p.x / TILE_CELLS, -p.y / TILE_CELLS])
        .collect();
    Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::default(),
    )
    .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, positions)
    .with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, vec![Vec3::Z; points.len()])
    .with_inserted_attribute(Mesh::ATTRIBUTE_UV_0, uvs)
    .with_inserted_indices(Indices::U32(indices))
}

/// Blue with paler streaks running down it, broken at different lengths per
/// column; it repeats both ways, so it can scroll for ever.
pub(super) fn picture() -> Image {
    let mut pixels = Vec::with_capacity((WIDTH * HEIGHT * 4) as usize);
    for y in 0..HEIGHT {
        for x in 0..WIDTH {
            // A streak every few columns, each with its own length and place.
            let column = x / 4;
            let on_line = x % 4 == 1;
            let waves = 1.0 + (column * 7 % 3) as f32;
            let phase = (column * 37 % 11) as f32 / 11.0;
            let along = y as f32 / HEIGHT as f32 * waves + phase;
            let streak = if on_line {
                (0.5 + 0.5 * (along * std::f32::consts::TAU).sin()).powi(3)
            } else {
                0.0
            };
            let mix = |a: u8, b: u8| (f32::from(a) + (f32::from(b) - f32::from(a)) * streak) as u8;
            let alpha = BLUE_ALPHA + (STREAK_ALPHA - BLUE_ALPHA) * streak;
            pixels.extend([
                mix(BLUE[0], STREAK[0]),
                mix(BLUE[1], STREAK[1]),
                mix(BLUE[2], STREAK[2]),
                (alpha * 255.0) as u8,
            ]);
        }
    }
    let mut image = Image::new(
        Extent3d {
            width: WIDTH,
            height: HEIGHT,
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        pixels,
        TextureFormat::Rgba8UnormSrgb,
        RenderAssetUsages::RENDER_WORLD,
    );
    image.sampler = ImageSampler::Descriptor(ImageSamplerDescriptor {
        address_mode_u: ImageAddressMode::Repeat,
        address_mode_v: ImageAddressMode::Repeat,
        ..ImageSamplerDescriptor::linear()
    });
    image
}
