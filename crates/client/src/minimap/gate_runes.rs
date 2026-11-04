//! A World Gate's code on the full map's hover box: drawn from the shared
//! rune strokes (`shared::hall::RUNE_STROKES`), never a font, so it looks
//! as it does on the gate.

use bevy::asset::RenderAssetUsages;
use bevy::prelude::*;
use bevy::render::render_resource::{Extent3d, TextureDimension, TextureFormat};
use shared::hall::{
    CODE_LENGTH, GLYPH_HEIGHT, GLYPH_WIDTH, GateCode, GatePlace, RUNE_STROKES, SEPARATOR_STROKES,
    Stroke,
};
use shared::protocol::LevelKind;

use super::canvas::Canvas;
use crate::common::{CurrentDungeon, NO_QUALITY_NAME};

/// The picture's height in interface pixels, and how many of its own pixels
/// each is drawn with, so it stays sharp.
pub(super) const RUNES_HEIGHT: f32 = 22.0;
const SHARPNESS: u32 = 2;
/// Grid steps between runes, the separator's width, and a stroke's width.
const GAP: f32 = 1.4;
const SEPARATOR: f32 = 2.0;
const STROKE: f32 = 0.8;
/// Space round the runes, in grid steps.
const MARGIN: f32 = 1.0;

/// The gate on this cell of the Hall, if the player is there and one stands
/// there.
pub(super) fn hovered_gate(dungeon: &CurrentDungeon, (x, y): (f64, f64)) -> Option<GateCode> {
    let LevelKind::Hall { origin } = dungeon.info.as_ref()?.kind else {
        return None;
    };
    let (x, y) = ((x + 0.5).floor() as i64, (y + 0.5).floor() as i64);
    GatePlace::at(x + origin, y).map(GatePlace::code)
}

/// How wide the picture is against its height.
pub(super) fn aspect() -> f32 {
    let steps = CODE_LENGTH as f32 * f32::from(GLYPH_WIDTH)
        + CODE_LENGTH as f32 * GAP
        + SEPARATOR
        + 2.0 * MARGIN;
    steps / (f32::from(GLYPH_HEIGHT) + 2.0 * MARGIN)
}

/// The code as a picture, in the tooltip's ink on nothing.
pub(super) fn picture(code: GateCode) -> Image {
    let height = RUNES_HEIGHT as u32 * SHARPNESS;
    let step = height as f32 / (f32::from(GLYPH_HEIGHT) + 2.0 * MARGIN);
    let width = (height as f32 * aspect()).ceil() as u32;
    let mut pixels = vec![0; (width * height * 4) as usize];
    let mut canvas = Canvas {
        pixels: &mut pixels,
        width,
        height,
    };
    let ink = NO_QUALITY_NAME.to_srgba().to_u8_array_no_alpha();
    let mut left = MARGIN;
    for (i, &index) in code.0.iter().enumerate() {
        if i == 3 {
            let middle = left + (SEPARATOR - f32::from(GLYPH_WIDTH)) / 2.0;
            draw(&mut canvas, SEPARATOR_STROKES, middle, step, ink);
            left += SEPARATOR + GAP;
        }
        draw(
            &mut canvas,
            RUNE_STROKES[usize::from(index & 31)],
            left,
            step,
            ink,
        );
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

fn draw(canvas: &mut Canvas, strokes: &[Stroke], left: f32, step: f32, ink: [u8; 3]) {
    let half = STROKE * step / 2.0;
    for &[x0, y0, x1, y1] in strokes {
        let at = |x: u8, y: u8| Vec2::new(left + f32::from(x), MARGIN + f32::from(y)) * step;
        let (a, b) = (at(x0, y0), at(x1, y1));
        let centre = (a + b) / 2.0;
        let reach = (b - a).length() / 2.0 + half + 1.0;
        canvas.shape((centre.x, centre.y), reach, ink, |x, y| {
            // How far from the stroke's line, less its half width.
            let p = Vec2::new(x, y);
            let along = b - a;
            let t = ((p - a).dot(along) / along.length_squared().max(1e-6)).clamp(0.0, 1.0);
            p.distance(a + along * t) - half
        });
    }
}
