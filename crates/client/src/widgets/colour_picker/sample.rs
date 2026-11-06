//! A gradient made once at start, for showing that a swatch can be a picture
//! (Show Widgets).

use bevy::asset::{RenderAssetUsages, uuid_handle};
use bevy::prelude::*;
use bevy::render::render_resource::{Extent3d, TextureDimension, TextureFormat};

/// A red-to-blue gradient, made at start (`make_sample_gradient`).
pub const SAMPLE_GRADIENT: Handle<Image> = uuid_handle!("3f6a1c2e-9b4d-4e7a-8c21-5d0b7e9f4a16");

pub(in crate::widgets) fn make_sample_gradient(mut images: ResMut<Assets<Image>>) {
    const SIDE: u32 = 16;
    let bytes = (0..SIDE * SIDE)
        .flat_map(|at| {
            let across = (at % SIDE) * 255 / (SIDE - 1);
            [255 - across as u8, 60, across as u8, 255]
        })
        .collect();
    let picture = Image::new(
        Extent3d {
            width: SIDE,
            height: SIDE,
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        bytes,
        TextureFormat::Rgba8UnormSrgb,
        RenderAssetUsages::default(),
    );
    let _ = images.insert(&SAMPLE_GRADIENT, picture);
}
