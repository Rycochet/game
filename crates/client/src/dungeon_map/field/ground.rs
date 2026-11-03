//! The flat ground of a field's cell, ruined fields included.

use bevy::prelude::*;

use super::super::town::{self, GRASS_TILE, PLOUGHED_TILE};
use super::super::walls::{Layers, Scene};
use super::{HALF_CELL, RUINED_TILES, Stage};

/// What the ground of a cell of a ruined field has gone back to: mostly grass,
/// some still ploughed, some (`None`) the land's own. All clients agree.
pub(super) fn ruined_tile(x: i64, y: i64) -> Option<usize> {
    match RUINED_TILES.at(&[x, y]) % 100 {
        0..20 => None,
        20..40 => Some(PLOUGHED_TILE),
        _ => Some(GRASS_TILE),
    }
}

/// The flat ground of a field's cell. False if it is the land's own, to be
/// drawn as any outside ground is.
pub fn build_field(layers: &mut Layers, scene: &Scene, (x, y): (i64, i64)) -> bool {
    let tile = if scene.ruined {
        ruined_tile(x, y)
    } else {
        let field = scene.fields.iter().find(|field| field.contains(x, y));
        Some(match field.map(Stage::of) {
            Some(Stage::Ploughed) => PLOUGHED_TILE,
            _ => GRASS_TILE,
        })
    };
    let Some(tile) = tile else {
        return false;
    };
    let patch = Rect::from_corners(Vec2::splat(-HALF_CELL), Vec2::splat(HALF_CELL));
    let layer = town::flat_layer(scene, x, y);
    town::flat_patch(layers, scene, layer, town::cell_centre(x, y), patch, tile);
    true
}
