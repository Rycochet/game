//! The pictures drawn for the panels: the person, the bin and the ground.

use bevy::asset::RenderAssetUsages;
use bevy::prelude::*;
use bevy::render::render_resource::{Extent3d, TextureDimension, TextureFormat};

/// Someone's head and shoulders, drawn here so there is no file to ship for it.
pub(super) fn person_icon() -> Image {
    const SIZE: usize = 32;
    const INK_BYTES: [u8; 4] = [60, 40, 20, 255];
    let mut bytes = vec![0; SIZE * SIZE * 4];
    let middle = (SIZE as f32 - 1.0) / 2.0;
    for y in 0..SIZE {
        for x in 0..SIZE {
            let (across, down) = (x as f32 - middle, y as f32);
            let head = across.hypot(down - 10.0) < 6.5;
            let shoulders = across.hypot(down - 30.0) < 12.5 && down < 28.0;
            if head || shoulders {
                bytes[(y * SIZE + x) * 4..][..4].copy_from_slice(&INK_BYTES);
            }
        }
    }
    Image::new(
        Extent3d {
            width: SIZE as u32,
            height: SIZE as u32,
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        bytes,
        TextureFormat::Rgba8UnormSrgb,
        RenderAssetUsages::default(),
    )
}

/// A waste bin, drawn here so there is no file to ship for it.
pub(super) fn bin_icon() -> Image {
    const SIZE: usize = 32;
    const INK_BYTES: [u8; 4] = [60, 40, 20, 255];
    let mut bytes = vec![0; SIZE * SIZE * 4];
    for y in 0..SIZE {
        for x in 0..SIZE {
            let (x, y) = (x as i32, y as i32);
            let handle = (13..19).contains(&x) && (4..6).contains(&y);
            let lid = (6..26).contains(&x) && (6..9).contains(&y);
            // Narrowing towards the bottom, with slots down it.
            let inset = (y - 10) / 5;
            let body = (10..28).contains(&y) && (8 + inset..24 - inset).contains(&x);
            let slot = (13..25).contains(&y) && (x == 12 || x == 15 || x == 16 || x == 19);
            if handle || lid || (body && !slot) {
                bytes[(y as usize * SIZE + x as usize) * 4..][..4].copy_from_slice(&INK_BYTES);
            }
        }
    }
    Image::new(
        Extent3d {
            width: SIZE as u32,
            height: SIZE as u32,
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        bytes,
        TextureFormat::Rgba8UnormSrgb,
        RenderAssetUsages::default(),
    )
}

/// A hand with an arrow down below it, drawn here so there is no file to ship for it.
pub(super) fn drop_icon() -> Image {
    const SIZE: usize = 32;
    const INK_BYTES: [u8; 4] = [60, 40, 20, 255];
    let mut bytes = vec![0; SIZE * SIZE * 4];
    for y in 0..SIZE {
        for x in 0..SIZE {
            let (x, y) = (x as i32, y as i32);
            // Four fingers pointing down from the palm, with gaps, and a thumb.
            let palm = (9..23).contains(&x) && (2..10).contains(&y);
            let finger = (9..23).contains(&x) && (10..17).contains(&y) && (x - 9) % 4 != 3;
            let thumb = (5..9).contains(&x) && (5..11).contains(&y);
            let shaft = (14..18).contains(&x) && (19..25).contains(&y);
            let across = (x as f32 - 15.5).abs();
            let head = (25..31).contains(&y) && across < (31 - y) as f32 + 0.5;
            if palm || finger || thumb || shaft || head {
                bytes[(y as usize * SIZE + x as usize) * 4..][..4].copy_from_slice(&INK_BYTES);
            }
        }
    }
    Image::new(
        Extent3d {
            width: SIZE as u32,
            height: SIZE as u32,
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        bytes,
        TextureFormat::Rgba8UnormSrgb,
        RenderAssetUsages::default(),
    )
}
