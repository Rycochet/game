//! Doors and windows on a building's walls.

use bevy::prelude::*;
use shared::dungeon::{Cell, Level};
use shared::site::BuildingLook;

use super::super::WALL_HEIGHT;
use super::super::walls::{Layer, Layers, Scene};
use super::{
    FIRST_DOOR_TILE, HALF_CELL, LIT_WINDOW_TILE, OVERLAY_LIFT, Piece, WALL_OUT, WINDOW_HALF,
    WINDOW_SILL, WINDOW_TILE, WINDOW_TOP, atlas_uv, cell_centre, emit, emit_as_given, face_layer,
    kind_index, layer_colour, side_shade, upright,
};

/// Which way a town's window faces, as a grid step: to the first walkable cell
/// beside it. `None` if nothing is, and then it isn't drawn.
pub(in crate::dungeon_map) fn window_faces(
    level: &Level,
    (x, y): (i64, i64),
) -> Option<(i64, i64)> {
    [(0, -1), (1, 0), (0, 1), (-1, 0)]
        .into_iter()
        .find(|&(dx, dy)| level.is_walkable(x + dx, y + dy))
}

/// The door or window on a wall cell, if it has one.
pub(super) fn build_door_or_window(
    layers: &mut Layers,
    scene: &Scene,
    building: &BuildingLook,
    (x, y): (i64, i64),
    hides: bool,
) {
    let level = scene.level;
    let cell = level.cell(x, y);
    let (faces, door) = match cell {
        Cell::BuildingDoor if building.door == (x, y) => (building.faces, true),
        Cell::BlindWindow => {
            let Some(faces) = window_faces(level, (x, y)) else {
                return;
            };
            (faces, false)
        }
        _ => return,
    };
    let normal = Vec3::new(faces.0 as f32, 0.0, faces.1 as f32);
    let along = Vec3::new(faces.1 as f32, 0.0, -faces.0 as f32);
    // Pictures read left to right seen from the front.
    let middle = cell_centre(x, y) + normal * (WALL_OUT + OVERLAY_LIFT);
    let half = if door { HALF_CELL } else { WINDOW_HALF };
    let piece = Piece {
        from: middle - along * half,
        to: middle + along * half,
        normal,
        across: (0.0, 1.0),
    };
    let outer = (x + faces.0, y + faces.1);
    let Some(layer) = face_layer(scene, (x, y), outer, &piece, hides) else {
        return;
    };
    if door {
        let tile = FIRST_DOOR_TILE + kind_index(building);
        upright(layers, scene, layer, &piece, (0.0, WALL_HEIGHT), tile);
    } else {
        let on = layer != Layer::Memory && scene.lit_buildings.contains(&building.id);
        let (tile, colour) = if on {
            // Giving light, so it doesn't take any.
            (LIT_WINDOW_TILE, Vec3::ONE)
        } else {
            (WINDOW_TILE, layer_colour(layer) * side_shade(normal))
        };
        let draw = if on { emit_as_given } else { emit };
        draw(
            layers,
            scene,
            layer,
            [
                piece.from + Vec3::Y * WINDOW_SILL,
                piece.to + Vec3::Y * WINDOW_SILL,
                piece.to + Vec3::Y * WINDOW_TOP,
                piece.from + Vec3::Y * WINDOW_TOP,
            ],
            normal,
            [
                atlas_uv(tile, 0.0, 1.0),
                atlas_uv(tile, 1.0, 1.0),
                atlas_uv(tile, 1.0, 0.0),
                atlas_uv(tile, 0.0, 0.0),
            ],
            [colour; 4],
        );
    }
}
