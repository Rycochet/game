//! A box of a wall's material.

use bevy::prelude::*;

use super::super::town;
use super::super::walls::{Layer, Layers, Scene};
use super::{Build, SIDE_SHADE};
use crate::dungeon_map::CELL_SIZE;

/// A box from `low` to `high` in a material's picture (its bottom isn't
/// drawn), each face showing as much of the picture as it is big, up to a cell.
pub(super) fn block(
    layers: &mut Layers,
    scene: &Scene,
    layer: Layer,
    build: &Build,
    (low, high): (Vec3, Vec3),
) {
    let (l, h) = (low, high);
    let size = h - l;
    let share = |length: f32| (length / CELL_SIZE).clamp(0.05, 1.0);
    let uvs = |across: f32, up: f32| {
        let (u, v) = (share(across), share(up));
        [(0.0, v), (u, v), (u, 0.0), (0.0, 0.0)].map(|(u, v)| town::atlas_uv(build.tile, u, v))
    };
    let top = town::layer_colour(layer) * build.shade;
    let side = top * SIDE_SHADE;
    let at = Vec3::new;
    let faces = [
        (
            [
                at(l.x, h.y, h.z),
                at(h.x, h.y, h.z),
                at(h.x, h.y, l.z),
                at(l.x, h.y, l.z),
            ],
            Vec3::Y,
            top,
            uvs(size.x, size.z),
        ),
        (
            [
                at(l.x, l.y, h.z),
                at(h.x, l.y, h.z),
                at(h.x, h.y, h.z),
                at(l.x, h.y, h.z),
            ],
            Vec3::Z,
            side,
            uvs(size.x, size.y),
        ),
        (
            [
                at(h.x, l.y, l.z),
                at(l.x, l.y, l.z),
                at(l.x, h.y, l.z),
                at(h.x, h.y, l.z),
            ],
            Vec3::NEG_Z,
            side,
            uvs(size.x, size.y),
        ),
        (
            [
                at(h.x, l.y, h.z),
                at(h.x, l.y, l.z),
                at(h.x, h.y, l.z),
                at(h.x, h.y, h.z),
            ],
            Vec3::X,
            side,
            uvs(size.z, size.y),
        ),
        (
            [
                at(l.x, l.y, l.z),
                at(l.x, l.y, h.z),
                at(l.x, h.y, h.z),
                at(l.x, h.y, l.z),
            ],
            Vec3::NEG_X,
            side,
            uvs(size.z, size.y),
        ),
    ];
    for (corners, normal, colour, uvs) in faces {
        town::emit(layers, scene, layer, corners, normal, uvs, [colour; 4]);
    }
}
