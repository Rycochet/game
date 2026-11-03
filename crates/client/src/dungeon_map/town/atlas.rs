//! Where things are in the town atlas: roads, walls, roofs and door pictures.

use shared::dungeon::Cell;
use shared::site::{BuildingKind, BuildingLook, RoofKind, WallKind};

use super::super::CELL_SIZE;
use super::{
    ATLAS_TILE_PIXELS, COLUMNS, FIRST_DOOR_TILE, FIRST_ROOF_TILE, FIRST_WALL_TILE, GARDEN_TILE,
    ROWS,
};

/// Picture coordinates in a tile of the atlas, from coordinates in that tile.
pub(in crate::dungeon_map) fn atlas_uv(tile: usize, u: f32, v: f32) -> [f32; 2] {
    let (column, row) = (tile as f32 % COLUMNS, (tile as f32 / COLUMNS).floor());
    [
        (column + u.clamp(0.0, 1.0)) / COLUMNS,
        (row + v.clamp(0.0, 1.0)) / ROWS,
    ]
}

/// A tile's coordinates for a point `(east, south)` from a cell's centre.
pub(super) fn flat_uv(tile: usize, east: f32, south: f32) -> [f32; 2] {
    atlas_uv(tile, east / CELL_SIZE + 0.5, south / CELL_SIZE + 0.5)
}

pub(super) fn road_tile(cell: Cell) -> Option<usize> {
    match cell {
        Cell::DirtTrack => Some(0),
        Cell::GravelTrack => Some(1),
        Cell::WoodenBoards => Some(2),
        Cell::Cobblestone => Some(3),
        Cell::Flagstones => Some(4),
        Cell::DressedStone => Some(5),
        Cell::Garden => Some(GARDEN_TILE),
        _ => None,
    }
}

pub(super) fn wall_tile(kind: WallKind) -> usize {
    FIRST_WALL_TILE + WallKind::ALL.iter().position(|&k| k == kind).unwrap_or(0)
}

pub(super) fn roof_tile(kind: RoofKind) -> usize {
    FIRST_ROOF_TILE + RoofKind::ALL.iter().position(|&k| k == kind).unwrap_or(0)
}

/// Where a kind of building's door picture is in the atlas: left, top and
/// size, in pixels (the maps show it as a shop's sign).
pub fn door_picture(kind: BuildingKind) -> (u32, u32, u32) {
    let tile = (FIRST_DOOR_TILE + kind as usize) as u32;
    let (columns, size) = (COLUMNS as u32, ATLAS_TILE_PIXELS);
    ((tile % columns) * size, (tile / columns) * size, size)
}

pub(super) fn kind_index(building: &BuildingLook) -> usize {
    building.kind as usize
}
