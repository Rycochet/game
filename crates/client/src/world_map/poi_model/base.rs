//! The flat base under a poi.

use bevy::asset::RenderAssetUsages;
use bevy::mesh::PrimitiveTopology;
use bevy::prelude::*;
use std::f32::consts::TAU;

use super::{BASE_RIM, BASE_ROUND_SIDES, BASE_WHITE, REACH};

/// The flat base under a poi, lying at height 0: white with a thin black rim,
/// the size of a token. Octagonal with a flat side towards the camera in the
/// starting view for towns, otherwise round.
pub fn base_mesh(town: bool) -> Mesh {
    let (sides, turn) = if town {
        (8, TAU / 16.0)
    } else {
        (BASE_ROUND_SIDES, 0.0)
    };
    let corner = |i: usize, radius: f32| {
        let angle = i as f32 / sides as f32 * TAU + turn;
        [angle.cos() * radius, 0.0, angle.sin() * radius]
    };
    let white = [BASE_WHITE, BASE_WHITE, BASE_WHITE, 1.0];
    let black = [0.0, 0.0, 0.0, 1.0];
    let inner = REACH - BASE_RIM;
    let mut positions = Vec::new();
    let mut colours = Vec::new();
    for i in 0..sides {
        // Going round this way looks clockwise from above, so each triangle
        // is listed backwards to face up.
        let (a, b) = (corner(i, inner), corner(i + 1, inner));
        let (outer_a, outer_b) = (corner(i, REACH), corner(i + 1, REACH));
        positions.extend([[0.0; 3], b, a]);
        colours.extend([white; 3]);
        positions.extend([a, b, outer_b, a, outer_b, outer_a]);
        colours.extend([black; 6]);
    }
    let normals = vec![[0.0, 1.0, 0.0]; positions.len()];
    Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::default(),
    )
    .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, positions)
    .with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, normals)
    .with_inserted_attribute(Mesh::ATTRIBUTE_COLOR, colours)
}
