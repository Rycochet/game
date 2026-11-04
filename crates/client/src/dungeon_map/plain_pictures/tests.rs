use bevy::asset::RenderAssetUsages;
use bevy::render::render_resource::{Extent3d, TextureDimension};

use super::*;

/// A picture two tiles wide, the left tile black and white stripes, the right
/// tile red.
fn two_tiles() -> Image {
    let (width, height) = (TILE * 2, TILE);
    let mut data = Vec::new();
    for _y in 0..height {
        for x in 0..width {
            let pixel = if x >= TILE {
                [200, 0, 0, 255]
            } else if x % 2 == 0 {
                [0, 0, 0, 255]
            } else {
                [200, 200, 200, 255]
            };
            data.extend(pixel);
        }
    }
    Image::new(
        Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        data,
        TextureFormat::Rgba8UnormSrgb,
        RenderAssetUsages::default(),
    )
}

#[test]
fn each_tile_becomes_its_average() {
    let plain = plain_copy(&two_tiles());
    let data = plain.data.unwrap();
    let at = |x: u32, y: u32| {
        let i = ((y * TILE * 2 + x) * 4) as usize;
        [data[i], data[i + 1], data[i + 2], data[i + 3]]
    };
    assert_eq!(at(0, 0), [100, 100, 100, 255]);
    assert_eq!(at(TILE - 1, TILE - 1), [100, 100, 100, 255]);
    assert_eq!(at(TILE, 5), [200, 0, 0, 255]);
}
