//! Drawing the pad's small white pictures in code, so there are no files to
//! ship for them: shapes as tests of points, smoothed at their edges.

use bevy::asset::RenderAssetUsages;
use bevy::prelude::*;
use bevy::render::render_resource::{Extent3d, TextureDimension, TextureFormat};

/// The pictures' size, across and down, in pixels; shapes are drawn in these
/// units, `y` down.
pub(super) const SIZE: usize = 64;
/// Points tried in each pixel, across and down, to smooth the edges.
const SAMPLES: usize = 4;

/// A white picture, seen where `inside` holds and clear elsewhere, tinted by
/// the `ImageNode` showing it.
pub(super) fn picture(inside: impl Fn(Vec2) -> bool) -> Image {
    let mut bytes = vec![0; SIZE * SIZE * 4];
    for y in 0..SIZE {
        for x in 0..SIZE {
            let hits = (0..SAMPLES * SAMPLES)
                .filter(|sample| {
                    let (across, down) = (sample % SAMPLES, sample / SAMPLES);
                    let step = 1.0 / SAMPLES as f32;
                    inside(Vec2::new(
                        x as f32 + (across as f32 + 0.5) * step,
                        y as f32 + (down as f32 + 0.5) * step,
                    ))
                })
                .count();
            let alpha = (hits * 255 / (SAMPLES * SAMPLES)) as u8;
            bytes[(y * SIZE + x) * 4..][..4].copy_from_slice(&[255, 255, 255, alpha]);
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

/// Whether `at` is within `thickness / 2` of the line from `from` to `to`.
pub(super) fn on_line(at: Vec2, (from, to): (Vec2, Vec2), thickness: f32) -> bool {
    let along = to - from;
    let share = ((at - from).dot(along) / along.length_squared()).clamp(0.0, 1.0);
    at.distance(from + along * share) <= thickness / 2.0
}

/// How far `at` is outside a box with rounded corners (negative inside), from
/// its middle, half its size and the corners' radius.
pub(super) fn outside_box(at: Vec2, middle: Vec2, half: Vec2, radius: f32) -> f32 {
    let beyond = (at - middle).abs() - half + Vec2::splat(radius);
    beyond.max(Vec2::ZERO).length() + beyond.x.max(beyond.y).min(0.0) - radius
}
