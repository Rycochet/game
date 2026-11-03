//! A bag of a look, with the rope round its neck.

use shared::ground::Load;
use std::f32::consts::TAU;

use super::{
    CANVAS, FOLD_SHADE, Model, RINGS, ROPE, ROPE_SEGMENTS, ROPE_SIDES, SEGMENTS, TIED_AT,
    WEAVE_ROUND, WEAVE_UP, on_curve, profile, size_of,
};

/// A bag of this look. Each variety has its own lumps and slumps its own way.
pub(in crate::ground) fn bag(load: Load, variety: u8) -> Model {
    let size = size_of(load);
    let points = profile(load);
    let phase = variety as f32 * 2.1 + 0.4;
    let lean_way = (phase.cos(), phase.sin());
    // Where a height is pushed to by the slump: more towards the top.
    let leaning = |up: f32| {
        let by = size.lean * up * up;
        (lean_way.0 * by, lean_way.1 * by)
    };
    let mut model = Model::default();
    for ring in 0..=RINGS {
        let along = ring as f32 / RINGS as f32;
        let (across, up) = on_curve(&points, along);
        let (shift_x, shift_z) = leaning(up);
        // Lumps only on the belly, and pleats where it is gathered.
        let belly = (up / TIED_AT).min(1.0);
        let belly = (belly * std::f32::consts::PI).sin().max(0.0);
        let frill = ((up - TIED_AT) / (1.0 - TIED_AT)).clamp(0.0, 1.0);
        for segment in 0..=SEGMENTS {
            // The last point round is the first again, with its own picture place.
            let round = (segment % SEGMENTS) as f32 / SEGMENTS as f32 * TAU;
            let lumps = 1.0
                + belly
                    * (0.1 * (2.0 * round + phase).sin()
                        + 0.07 * (3.0 * round + phase * 2.0).sin()
                        + 0.03 * (7.0 * round + phase * 3.0).sin());
            let pleats = 1.0 + frill * 0.22 * (9.0 * round + phase).sin();
            let radius = size.radius * across * lumps * pleats;
            let fold = 0.5 + 0.5 * (5.0 * round + phase).sin() * (1.0 - belly);
            let shade = 1.0 - (1.0 - FOLD_SHADE) * fold.clamp(0.0, 1.0);
            model.positions.push([
                radius * round.cos() + shift_x,
                size.height * up,
                radius * round.sin() + shift_z,
            ]);
            model.normals.push([0.0; 3]);
            model.uvs.push([
                segment as f32 / SEGMENTS as f32 * WEAVE_ROUND,
                along * WEAVE_UP,
            ]);
            model
                .colours
                .push([CANVAS[0] * shade, CANVAS[1] * shade, CANVAS[2] * shade, 1.0]);
        }
    }
    let row = SEGMENTS + 1;
    for ring in 0..RINGS {
        for segment in 0..SEGMENTS {
            let low = ring * row + segment;
            let high = low + row;
            model.add_triangle(low, high, high + 1);
            model.add_triangle(low, high + 1, low + 1);
        }
    }
    model.smooth_normals(0, 0);
    // Both ends of the seam share a normal, so it doesn't show.
    for ring in 0..=RINGS {
        let (first, last) = (ring * row, ring * row + SEGMENTS);
        let joined = model.normals[first];
        model.normals[last] = joined;
    }
    add_rope(&mut model, load, leaning);
    model
}

/// The rope round the neck.
pub(super) fn add_rope(model: &mut Model, load: Load, leaning: impl Fn(f32) -> (f32, f32)) {
    let size = size_of(load);
    let (shift_x, shift_z) = leaning(TIED_AT);
    let thickness = (size.height * 0.06).clamp(0.15, 0.7);
    let round = size.radius * 0.2 + thickness * 0.6;
    let middle_up = size.height * TIED_AT;
    let first_point = model.positions.len();
    let first_index = model.indices.len();
    for along in 0..ROPE_SEGMENTS {
        let along = along as f32 / ROPE_SEGMENTS as f32 * TAU;
        let (out_x, out_z) = (along.cos(), along.sin());
        for side in 0..ROPE_SIDES {
            let side = side as f32 / ROPE_SIDES as f32 * TAU;
            let radius = round + thickness * side.cos();
            model.positions.push([
                radius * out_x + shift_x,
                middle_up + thickness * side.sin(),
                radius * out_z + shift_z,
            ]);
            model.normals.push([0.0; 3]);
            model.uvs.push([0.0, 0.0]);
            model.colours.push([ROPE[0], ROPE[1], ROPE[2], 1.0]);
        }
    }
    let point = |along: usize, side: usize| {
        first_point + (along % ROPE_SEGMENTS) * ROPE_SIDES + side % ROPE_SIDES
    };
    for along in 0..ROPE_SEGMENTS {
        for side in 0..ROPE_SIDES {
            let (a, b) = (point(along, side), point(along + 1, side));
            let (c, d) = (point(along + 1, side + 1), point(along, side + 1));
            model.add_triangle(a, c, b);
            model.add_triangle(a, d, c);
        }
    }
    model.smooth_normals(first_point, first_index);
}
