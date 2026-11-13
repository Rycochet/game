//! A planet's name in runes as a picture, for the planet chooser: drawn from
//! the shared rune strokes (`shared::hall::RUNE_STROKES`), never a font
//! (owner), so it looks as the code does on the gates.

use bevy::asset::RenderAssetUsages;
use bevy::prelude::*;
use bevy::render::render_resource::{Extent3d, TextureDimension, TextureFormat};
use shared::hall::{
    CODE_LENGTH, GLYPH_HEIGHT, GLYPH_WIDTH, GateCode, RUNE_STROKES, SEPARATOR_STROKES, Stroke,
};

use crate::widgets::INK;

/// Pixels a grid step, so the picture stays sharp at a line's height.
const STEP: f32 = 6.0;
/// Grid steps between runes, the separator's width, a stroke's width and the
/// space round them.
const GAP: f32 = 1.4;
const SEPARATOR: f32 = 2.0;
const STROKE: f32 = 0.8;
const MARGIN: f32 = 0.6;

/// The code's runes in ink on nothing.
pub(super) fn picture(code: GateCode) -> Image {
    let steps_wide = CODE_LENGTH as f32 * (f32::from(GLYPH_WIDTH) + GAP) + SEPARATOR + 2.0 * MARGIN;
    let width = (steps_wide * STEP).ceil() as u32;
    let height = ((f32::from(GLYPH_HEIGHT) + 2.0 * MARGIN) * STEP).ceil() as u32;
    let mut pixels = vec![0u8; (width * height * 4) as usize];
    let ink = INK.to_srgba().to_u8_array_no_alpha();
    let mut left = MARGIN;
    for (i, &index) in code.0.iter().enumerate() {
        if i == 3 {
            let middle = left + (SEPARATOR - f32::from(GLYPH_WIDTH)) / 2.0;
            draw(&mut pixels, width, SEPARATOR_STROKES, middle, ink);
            left += SEPARATOR + GAP;
        }
        let strokes = RUNE_STROKES[usize::from(index & 31)];
        draw(&mut pixels, width, strokes, left, ink);
        left += f32::from(GLYPH_WIDTH) + GAP;
    }
    Image::new(
        Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        pixels,
        TextureFormat::Rgba8UnormSrgb,
        RenderAssetUsages::RENDER_WORLD | RenderAssetUsages::MAIN_WORLD,
    )
}

/// Each stroke as a line with rounded ends, its edge smoothed over a pixel.
fn draw(pixels: &mut [u8], width: u32, strokes: &[Stroke], left: f32, ink: [u8; 3]) {
    let height = pixels.len() as u32 / 4 / width;
    let half = STROKE * STEP / 2.0;
    for &[x0, y0, x1, y1] in strokes {
        let at = |x: u8, y: u8| Vec2::new(left + f32::from(x), MARGIN + f32::from(y)) * STEP;
        let (a, b) = (at(x0, y0), at(x1, y1));
        let low = a.min(b) - Vec2::splat(half + 1.0);
        let high = a.max(b) + Vec2::splat(half + 1.0);
        let xs = (low.x.max(0.0) as u32)..(high.x.ceil() as u32).min(width);
        for y in (low.y.max(0.0) as u32)..(high.y.ceil() as u32).min(height) {
            for x in xs.clone() {
                let p = Vec2::new(x as f32 + 0.5, y as f32 + 0.5);
                let along = b - a;
                let t = ((p - a).dot(along) / along.length_squared().max(1e-6)).clamp(0.0, 1.0);
                let cover = (half + 0.5 - p.distance(a + along * t)).clamp(0.0, 1.0);
                let pixel = ((y * width + x) * 4) as usize;
                let alpha = (cover * 255.0) as u8;
                if alpha > pixels[pixel + 3] {
                    pixels[pixel..pixel + 3].copy_from_slice(&ink);
                    pixels[pixel + 3] = alpha;
                }
            }
        }
    }
}
