//! A bag's outlines, and a sparkle's star.

use std::f32::consts::TAU;

use super::{HOVER_OUTLINE_WIDTH, Model, OUTLINE_WIDTH};

/// A bag's outline shape.
pub(in crate::ground) fn outline(model: &Model) -> Model {
    model.grown(OUTLINE_WIDTH)
}

/// The wider outline of a bag the pointer is on.
pub(in crate::ground) fn hover_outline(model: &Model) -> Model {
    model.grown(HOVER_OUTLINE_WIDTH)
}

/// A flat star with four long points, facing +z, about its middle: what a sparkle is
/// made of. `grow` makes it larger all round, for its outline.
pub(in crate::ground) fn star(long: f32, short: f32, grow: f32) -> Model {
    let mut model = Model {
        positions: vec![[0.0; 3]],
        normals: vec![[0.0, 0.0, 1.0]],
        uvs: vec![[0.0, 0.0]],
        colours: vec![[1.0; 4]],
        indices: Vec::new(),
    };
    const POINTS: usize = 8;
    for point in 0..POINTS {
        let angle = point as f32 / POINTS as f32 * TAU;
        let radius = if point % 2 == 0 { long } else { short } + grow;
        model
            .positions
            .push([radius * angle.cos(), radius * angle.sin(), 0.0]);
        model.normals.push([0.0, 0.0, 1.0]);
        model.uvs.push([0.0, 0.0]);
        model.colours.push([1.0; 4]);
    }
    for point in 0..POINTS {
        model.add_triangle(0, point + 1, (point + 1) % POINTS + 1);
    }
    model
}
