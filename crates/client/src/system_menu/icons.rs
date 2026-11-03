//! Connect's small pictures, drawn here so there are no files to ship for them:
//! a padlock, shut when a certificate is saved, and a bin.

use bevy::asset::RenderAssetUsages;
use bevy::prelude::*;
use bevy::render::render_resource::{Extent3d, TextureDimension, TextureFormat};

const SIZE: usize = 64;
const INK: [u8; 4] = [60, 40, 20, 255];

/// The pictures, made once.
#[derive(Resource, Clone)]
pub(super) struct ConnectIcons {
    pub(super) shut: Handle<Image>,
    pub(super) open: Handle<Image>,
    pub(super) bin: Handle<Image>,
}

impl ConnectIcons {
    pub(super) fn new(images: &mut Assets<Image>) -> Self {
        Self {
            shut: images.add(padlock(true)),
            open: images.add(padlock(false)),
            bin: images.add(bin()),
        }
    }
}

/// An image where `inked(x, y)` says which pixels are drawn (from the top left).
fn draw(inked: impl Fn(f32, f32) -> bool) -> Image {
    let mut bytes = vec![0; SIZE * SIZE * 4];
    for y in 0..SIZE {
        for x in 0..SIZE {
            if inked(x as f32 + 0.5, y as f32 + 0.5) {
                bytes[(y * SIZE + x) * 4..][..4].copy_from_slice(&INK);
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

/// A padlock's body and shackle; open, the shackle is lifted out on one side.
fn padlock(shut: bool) -> Image {
    draw(|x, y| {
        let body = (14.0..=50.0).contains(&x) && (30.0..=56.0).contains(&y);
        let keyhole = (x - 32.0).hypot(y - 40.0) < 4.0
            || ((30.0..34.0).contains(&x) && (40.0..49.0).contains(&y));
        let lift = if shut { 0.0 } else { 8.0 };
        // A ring 5 pixels thick round (32, 22), its top half, and legs down to the body.
        let (cx, cy) = (32.0, 22.0 - lift);
        let ring = (11.0..=16.0).contains(&(x - cx).hypot(y - cy)) && y <= cy;
        let leg = |at: f32| (at - 2.5..=at + 2.5).contains(&x) && (cy..30.0).contains(&y);
        let right_leg = if shut {
            leg(cx + 13.5)
        } else {
            leg(cx + 13.5) && y < cy + 4.0
        };
        (body && !keyhole) || ring || leg(cx - 13.5) || right_leg
    })
}

/// A bin with a lid and a handle.
fn bin() -> Image {
    draw(|x, y| {
        let lid = (12.0..=52.0).contains(&x) && (14.0..=19.0).contains(&y);
        let handle = (26.0..=38.0).contains(&x)
            && (8.0..=14.0).contains(&y)
            && !((30.0..=34.0).contains(&x) && y > 10.0);
        // Narrowing towards the bottom, with two slots.
        let inset = (y - 22.0) / 34.0 * 4.0;
        let can = (22.0..=56.0).contains(&y) && (16.0 + inset..=48.0 - inset).contains(&x);
        let slot = [26.0, 38.0]
            .iter()
            .any(|&at: &f32| (at - 1.5..=at + 1.5).contains(&x) && (28.0..=50.0).contains(&y));
        lid || handle || (can && !slot)
    })
}
