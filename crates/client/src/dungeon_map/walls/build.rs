//! Building a wall or an opening's cell.

use bevy::prelude::*;
use shared::dungeon::Cell;

use super::super::theme::Shape;
use super::{
    HALF_CELL, Layer, Layers, OPENING_HALF, Patch, QUARTERS, Scene, half_thickness, units_to_size,
};

/// One cell being drawn. `joins` says which sides (north, east, south, west)
/// the wall runs out to: light doesn't blend across it there.
pub(super) struct Tile<'a> {
    pub(super) scene: &'a Scene<'a>,
    pub(super) x: i64,
    pub(super) y: i64,
    pub(super) centre: Vec3,
    pub(super) joins: [bool; 4],
}

/// Draws the wall at `(x, y)`, whose cell is remembered.
pub fn build_wall(layers: &mut Layers, scene: &Scene, (x, y): (i64, i64)) {
    let Some(shape) = scene.level.wall_shape(x, y) else {
        return;
    };
    let joins = [shape.north, shape.east, shape.south, shape.west];
    let reach = shape.reach().map(units_to_size);
    let tile = Tile::new(scene, (x, y), joins);
    let turned_away = tile.turned_away(reach);
    for quarter in QUARTERS {
        tile.build_wall_quarter(layers, quarter, reach, turned_away);
    }
    // A wall's curve lets the camera see in under it from the side, as far as
    // its foot. See-through walls keep their dark band, and the far side of a
    // wall stays dark.
    if !turned_away.contains(&true) {
        for quarter in QUARTERS {
            if tile.quarter_shows(scene.sight, quarter) {
                tile.fill_foot(layers, quarter, reach);
            }
        }
    }
}

/// Draws the door or window at `(x, y)`, whose cell is remembered: its floor
/// and the wall with a hole in it.
pub fn build_opening(layers: &mut Layers, scene: &Scene, (x, y): (i64, i64)) {
    let east_west = scene.level.opening_runs_east_west(x, y);
    let joins = [!east_west, east_west, !east_west, east_west];
    let tile = Tile::new(scene, (x, y), joins);
    let in_sight = scene.sight.is_seen(x, y);
    let floor_layer = if in_sight {
        Layer::Solid
    } else {
        Layer::Memory
    };
    let window = scene.level.cell(x, y) == Cell::Window;
    let half = half_thickness(scene.shape, 0.0);
    // A framed window's sill stands right across; a door's wall only beside
    // the hole. Columns barely touch the floor, so it runs on under them, but
    // it fades in from the sides like a door to meet the walls beside it.
    let along = if window && scene.shape == Shape::Built {
        0.0
    } else {
        OPENING_HALF
    };
    let wall = if east_west {
        Patch {
            east: (along, HALF_CELL),
            south: (0.0, half),
        }
    } else {
        Patch {
            east: (0.0, half),
            south: (along, HALF_CELL),
        }
    };
    let columns = window && scene.shape == Shape::Natural;
    let foot: &[Patch] = if columns { &[] } else { &[wall] };
    for quarter in QUARTERS {
        tile.floor_quarter(layers, floor_layer, quarter, foot, &[wall]);
    }
    tile.build_opening_wall(layers, east_west, window, in_sight);
}
