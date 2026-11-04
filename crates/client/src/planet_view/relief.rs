//! The globe's relief: the ground's slopes as a normal map, so the suns pick
//! out ranges and the line between day and night shows them, while the globe
//! itself stays round. Its alpha is the height, which the clouds read too.

use bevy::asset::RenderAssetUsages;
use bevy::image::{ImageAddressMode, ImageSampler, ImageSamplerDescriptor};
use bevy::prelude::*;
use bevy::render::render_resource::{Extent3d, TextureDimension, TextureFormat};

/// How many times steeper slopes are drawn than they are: real mountains are
/// far too small next to a planet to show at all.
const RELIEF: f32 = 20.0;
/// The height drawn as full alpha, in metres; higher is the same.
const HIGHEST: f32 = 9000.0;
/// Level ground, and nothing high.
const FLAT: [u8; 4] = [128, 128, 255, 0];

/// A level picture of this size, which wraps east-west. Linear, not sRGB:
/// its numbers are directions, not colours.
pub(super) fn new_relief_texture(size: UVec2) -> Image {
    let mut image = Image::new_fill(
        Extent3d {
            width: size.x,
            height: size.y,
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        &FLAT,
        TextureFormat::Rgba8Unorm,
        RenderAssetUsages::default(),
    );
    image.sampler = ImageSampler::Descriptor(ImageSamplerDescriptor {
        address_mode_u: ImageAddressMode::Repeat,
        ..ImageSamplerDescriptor::linear()
    });
    image
}

/// One row of the relief from the heights (metres, the sea level), for a
/// planet `circumference` metres round. Rows run from the north pole south.
pub(super) fn relief_row(
    heights: &[f32],
    size: UVec2,
    row: u32,
    circumference: f32,
    data: &mut [u8],
) {
    let (columns, rows) = (size.x as usize, size.y as usize);
    let row = row as usize;
    let at = |column: usize, row: usize| heights[row * columns + column];
    // A pixel's size on the ground: the same both ways at the equator, east to
    // west narrowing towards the poles.
    let latitude = ((row as f32 + 0.5) / rows as f32 - 0.5) * std::f32::consts::PI;
    let along = circumference / columns as f32;
    let east = along * latitude.cos().max(0.05);
    let (north, south) = (row.saturating_sub(1), (row + 1).min(rows - 1));
    let down = along * (south - north).max(1) as f32;
    for column in 0..columns {
        let (west, eastward) = ((column + columns - 1) % columns, (column + 1) % columns);
        let slope_east = (at(eastward, row) - at(west, row)) / (2.0 * east);
        let slope_south = (at(column, south) - at(column, north)) / down;
        let normal = Vec3::new(-RELIEF * slope_east, -RELIEF * slope_south, 1.0).normalize();
        let height = (at(column, row) / HIGHEST).clamp(0.0, 1.0);
        let byte = |value: f32| ((value * 0.5 + 0.5) * 255.0).round() as u8;
        let start = (row * columns + column) * 4;
        data[start..start + 4].copy_from_slice(&[
            byte(normal.x),
            byte(normal.y),
            byte(normal.z),
            (height * 255.0).round() as u8,
        ]);
    }
}
