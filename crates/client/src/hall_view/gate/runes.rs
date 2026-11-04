//! A gate's code cut along the band above its opening, on both faces, from
//! the shared rune strokes (`shared::hall::RUNE_STROKES`): never a font, so
//! every rune shown anywhere is the same.

use bevy::prelude::*;
use shared::hall::{
    CODE_LENGTH, GLYPH_HEIGHT, GLYPH_WIDTH, GateCode, RUNE_STROKES, SEPARATOR_STROKES, Stroke,
};

use super::{DEPTH, GateModel, RUNE_BAND, Rgb};

/// A rune's height, as a share of the band.
const RUNE_SHARE: f32 = 0.7;
/// Space between runes, and the separator's width, in grid steps.
const GAP_STEPS: f32 = 1.2;
const SEPARATOR_STEPS: f32 = 2.0;
/// A stroke's width, in grid steps, and how far it stands off the face.
const STROKE_STEPS: f32 = 0.7;
const STAND_OFF: f32 = 0.006;
/// The black outline round each stroke, in grid steps.
const OUTLINE_STEPS: f32 = 0.35;
const OUTLINE: Rgb = [0, 0, 0];
/// The runes come after the first 3, as `GateCode` shows them.
const BEFORE_SEPARATOR: usize = 3;

/// The code's runes in the band from `band.0` to `band.1` above the ground.
pub(super) fn build(model: &mut GateModel, code: GateCode, band: (f32, f32), colour: Rgb) {
    let step = RUNE_BAND * RUNE_SHARE / f32::from(GLYPH_HEIGHT);
    let rune = f32::from(GLYPH_WIDTH);
    let width = CODE_LENGTH as f32 * rune
        + (CODE_LENGTH as f32 - 1.0) * GAP_STEPS
        + SEPARATOR_STEPS
        + GAP_STEPS;
    let top = (band.0 + band.1) / 2.0 + f32::from(GLYPH_HEIGHT) * step / 2.0;
    let mut glyphs: Vec<(f32, &[Stroke])> = Vec::new();
    let mut x = -width / 2.0;
    for (i, &index) in code.0.iter().enumerate() {
        if i == BEFORE_SEPARATOR {
            glyphs.push((x + (SEPARATOR_STEPS - rune) / 2.0, SEPARATOR_STROKES));
            x += SEPARATOR_STEPS + GAP_STEPS;
        }
        glyphs.push((x, RUNE_STROKES[usize::from(index & 31)]));
        x += rune + GAP_STEPS;
    }
    for (left, strokes) in glyphs {
        for &stroke in strokes {
            // On the back the code reads the right way from behind too.
            for side in [1.0, -1.0] {
                inlay(model, stroke, left, top, step, side, colour);
            }
        }
    }
}

/// One stroke as a flat strip just off a face (`side` 1 the front, -1 the
/// back, mirrored).
fn inlay(
    model: &mut GateModel,
    [x0, y0, x1, y1]: Stroke,
    left: f32,
    top: f32,
    step: f32,
    side: f32,
    colour: Rgb,
) {
    let point = |x: u8, y: u8| {
        Vec2::new(
            (left + f32::from(x)) * step * side,
            top - f32::from(y) * step,
        )
    };
    let (from, to) = (point(x0, y0), point(x1, y1));
    let along = (to - from).try_normalize().unwrap_or(Vec2::X);
    let normal = Vec3::Z * side;
    // A black outline just behind the stroke, so it stands out from the stone.
    for (widen, off, colour) in [
        (OUTLINE_STEPS, STAND_OFF / 2.0, OUTLINE),
        (0.0, STAND_OFF, colour),
    ] {
        let half = (STROKE_STEPS / 2.0 + widen) * step;
        let across = along.perp() * half;
        let (a, b) = (from - along * half, to + along * half);
        let z = side * (DEPTH / 2.0 + off);
        let at = |p: Vec2| Vec3::new(p.x, p.y, z);
        let corners = [
            at(a - across),
            at(b - across),
            at(b + across),
            at(a + across),
        ];
        model.quad(corners, normal, colour);
    }
}
