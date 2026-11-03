//! Flat roads and gardens, and pieces of a flat tile.

use bevy::prelude::*;

use super::super::mesh::MEMORY_SHADE;
use super::super::walls::{Layer, Layers, Scene};
use super::{HALF_CELL, cell_centre, emit, flat_uv, road_tile};

/// The flat road or garden at `(x, y)`.
pub fn build_flat(layers: &mut Layers, scene: &Scene, (x, y): (i64, i64)) {
    let Some(tile) = road_tile(scene.level.cell(x, y)) else {
        return;
    };
    let layer = flat_layer(scene, x, y);
    let patch = Rect::from_corners(Vec2::splat(-HALF_CELL), Vec2::splat(HALF_CELL));
    flat_patch(layers, scene, layer, cell_centre(x, y), patch, tile);
}

pub(in crate::dungeon_map) fn flat_layer(scene: &Scene, x: i64, y: i64) -> Layer {
    if scene.sight.is_seen(x, y) {
        Layer::Solid
    } else {
        Layer::Memory
    }
}

pub(in crate::dungeon_map) fn layer_colour(layer: Layer) -> Vec3 {
    match layer {
        Layer::Memory => Vec3::splat(MEMORY_SHADE),
        _ => Vec3::ONE,
    }
}

/// A flat piece of a tile, as distances from `centre` (`x` east, `y` south).
pub(in crate::dungeon_map) fn flat_patch(
    layers: &mut Layers,
    scene: &Scene,
    layer: Layer,
    centre: Vec3,
    patch: Rect,
    tile: usize,
) {
    let corners = [
        (patch.min.x, patch.min.y),
        (patch.max.x, patch.min.y),
        (patch.max.x, patch.max.y),
        (patch.min.x, patch.max.y),
    ];
    emit(
        layers,
        scene,
        layer,
        corners.map(|(east, south)| centre + Vec3::new(east, 0.0, south)),
        Vec3::Y,
        corners.map(|(east, south)| flat_uv(tile, east, south)),
        [layer_colour(layer); 4],
    );
}
