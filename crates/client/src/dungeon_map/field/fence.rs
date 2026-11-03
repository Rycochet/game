//! The wood of a fence's or gate's cell.

use bevy::prelude::*;
use shared::dungeon::Cell;

use super::super::town::{self, FENCE_TILE};
use super::super::walls::{Layer, Layers, Scene};
use super::{
    FENCE_HEIGHT, GATE_END_HALF, GATE_POST, HALF_CELL, RAIL_HALF, RAIL_HEIGHTS, WOOD_SIDE_SHADE,
    post, rails, wood,
};

/// The wood of a fence's or gate's cell (nothing on any other cell).
pub fn build_fence(layers: &mut Layers, scene: &Scene, (x, y): (i64, i64)) {
    let level = scene.level;
    let cell = level.cell(x, y);
    if !matches!(cell, Cell::Fence | Cell::FieldGate) {
        return;
    }
    let layer = town::flat_layer(scene, x, y);
    let centre = town::cell_centre(x, y);
    let way = |(dx, dy): (i64, i64)| Vec3::new(dx as f32, 0.0, dy as f32);
    let sides = [(0, -1), (1, 0), (0, 1), (-1, 0)];
    if cell == Cell::Fence {
        post(layers, scene, layer, centre, FENCE_HEIGHT);
        for side in sides {
            if matches!(
                level.cell(x + side.0, y + side.1),
                Cell::Fence | Cell::FieldGate
            ) {
                let edge = centre + way(side) * HALF_CELL;
                rails(layers, scene, layer, centre, edge);
            }
        }
        return;
    }
    // A gate's cell: a post each side, and its leaf standing open at 45 degrees,
    // into the field.
    let inward = sides
        .into_iter()
        .find(|side| level.cell(x + side.0, y + side.1) == Cell::Field)
        .unwrap_or((0, 1));
    let along = way((inward.1, inward.0));
    for sign in [-1.0, 1.0] {
        let at = centre + along * sign * GATE_POST;
        post(layers, scene, layer, at, FENCE_HEIGHT);
        rails(layers, scene, layer, at, centre + along * sign * HALF_CELL);
    }
    let hinge = centre + along * GATE_POST;
    let open = (way(inward) - along).normalize();
    let end = hinge + open * GATE_POST * 2.0;
    for height in RAIL_HEIGHTS {
        slanted_rail(layers, scene, layer, hinge, end, height);
    }
    let thick = Vec3::new(GATE_END_HALF, 0.0, GATE_END_HALF);
    let (bottom, top) = (
        RAIL_HEIGHTS[0] - RAIL_HALF * 2.0,
        RAIL_HEIGHTS[1] + RAIL_HALF * 2.0,
    );
    wood(
        layers,
        scene,
        layer,
        end - thick + Vec3::Y * bottom,
        end + thick + Vec3::Y * top,
    );
}

/// A rail at `height` between two points on the ground, whichever way they lie:
/// its top and both sides.
pub(super) fn slanted_rail(
    layers: &mut Layers,
    scene: &Scene,
    layer: Layer,
    from: Vec3,
    to: Vec3,
    height: f32,
) {
    let uvs = [(0.1, 0.5), (0.9, 0.5), (0.9, 0.1), (0.1, 0.1)];
    let uvs = uvs.map(|(u, v)| town::atlas_uv(FENCE_TILE, u, v));
    let top = town::layer_colour(layer);
    let across = (to - from).cross(Vec3::Y).normalize() * RAIL_HALF;
    let up = Vec3::Y * RAIL_HALF;
    let (from, to) = (from + Vec3::Y * height, to + Vec3::Y * height);
    let faces = [
        (
            [
                from - across + up,
                to - across + up,
                to + across + up,
                from + across + up,
            ],
            Vec3::Y,
            top,
        ),
        (
            [
                from + across - up,
                to + across - up,
                to + across + up,
                from + across + up,
            ],
            across.normalize(),
            top * WOOD_SIDE_SHADE,
        ),
        (
            [
                from - across - up,
                to - across - up,
                to - across + up,
                from - across + up,
            ],
            -across.normalize(),
            top * WOOD_SIDE_SHADE,
        ),
    ];
    for (corners, normal, colour) in faces {
        town::emit(layers, scene, layer, corners, normal, uvs, [colour; 4]);
    }
}
