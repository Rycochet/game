//! What only oases have: grass round their pools, and the trees on it.

use bevy::prelude::*;
use shared::dungeon::Cell;

use super::CELL_SIZE;
use super::town::{self, FENCE_TILE, GRASS_TILE, HEDGE_TILE};
use super::walls::{Layer, Layers, Scene};

const HALF_CELL: f32 = CELL_SIZE / 2.0;

/// A tree: its trunk's height and half width, and its crown's half width and
/// depth, the crown sitting on the trunk.
const TRUNK_HEIGHT: f32 = CELL_SIZE * 1.1;
const TRUNK_HALF: f32 = CELL_SIZE * 0.08;
const CROWN_HALF: f32 = CELL_SIZE * 0.45;
const CROWN_DEPTH: f32 = CELL_SIZE * 0.35;

/// The sides of a block are darker than its top.
const SIDE_SHADE: f32 = 0.75;

/// The grass under a grass or tree cell, and the tree on a tree cell. False for
/// any other cell, which is drawn as usual.
pub fn build_grass_or_tree(layers: &mut Layers, scene: &Scene, (x, y): (i64, i64)) -> bool {
    let cell = scene.level.cell(x, y);
    if !matches!(cell, Cell::Grass | Cell::Tree) {
        return false;
    }
    let layer = town::flat_layer(scene, x, y);
    let centre = town::cell_centre(x, y);
    let patch = Rect::from_corners(Vec2::splat(-HALF_CELL), Vec2::splat(HALF_CELL));
    town::flat_patch(layers, scene, layer, centre, patch, GRASS_TILE);
    if cell == Cell::Tree {
        let trunk = Vec3::new(TRUNK_HALF, 0.0, TRUNK_HALF);
        let top = Vec3::Y * TRUNK_HEIGHT;
        block(
            layers,
            scene,
            layer,
            (centre - trunk, centre + trunk + top),
            FENCE_TILE,
        );
        let crown = Vec3::new(CROWN_HALF, 0.0, CROWN_HALF);
        let low = centre + top - Vec3::Y * CROWN_DEPTH * 0.5;
        let high = centre + top + Vec3::Y * CROWN_DEPTH * 0.5;
        block(
            layers,
            scene,
            layer,
            (low - crown, high + crown),
            HEDGE_TILE,
        );
    }
    true
}

/// A block from `low` to `high` in one tile of the town's picture: its top
/// and four sides (the bottom is never seen).
fn block(layers: &mut Layers, scene: &Scene, layer: Layer, (l, h): (Vec3, Vec3), tile: usize) {
    let uvs = [(0.1, 0.9), (0.9, 0.9), (0.9, 0.1), (0.1, 0.1)];
    let uvs = uvs.map(|(u, v)| town::atlas_uv(tile, u, v));
    let top = town::layer_colour(layer);
    let side = top * SIDE_SHADE;
    let at = Vec3::new;
    let faces = [
        (
            [
                at(l.x, h.y, l.z),
                at(h.x, h.y, l.z),
                at(h.x, h.y, h.z),
                at(l.x, h.y, h.z),
            ],
            Vec3::Y,
            top,
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
        ),
    ];
    for (corners, normal, colour) in faces {
        town::emit(layers, scene, layer, corners, normal, uvs, [colour; 4]);
    }
}
