//! The picture on the Quest Log's button: a list (three dots, each with a line
//! beside it), drawn here in the Character button's ink so there is no file to
//! ship for it.

use bevy::asset::RenderAssetUsages;
use bevy::prelude::*;
use bevy::render::render_resource::{Extent3d, TextureDimension, TextureFormat};

pub(super) fn list_icon() -> Image {
    const SIZE: usize = 32;
    const INK_BYTES: [u8; 4] = [60, 40, 20, 255];
    // Each row's middle, down from the top.
    const ROWS: [f32; 3] = [8.5, 16.0, 23.5];
    let mut bytes = vec![0; SIZE * SIZE * 4];
    for y in 0..SIZE {
        for x in 0..SIZE {
            let (across, down) = (x as f32 + 0.5, y as f32 + 0.5);
            let inked = ROWS.iter().any(|&row| {
                let dot = (across - 6.0).hypot(down - row) < 2.8;
                let line = (11.0..28.0).contains(&across) && (down - row).abs() < 2.0;
                dot || line
            });
            if inked {
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
