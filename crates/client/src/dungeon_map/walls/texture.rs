//! Noise and texture coordinates for wall faces and floors.

use crate::dungeon_map::liquid::LooksLiquid;
use shared::dungeon::{Cell, Level};

use super::super::CELL_SIZE;
use super::super::mesh::Surface;

/// A number from -1 to 1 that depends only on the numbers given: for looks,
/// so it needn't be the same as anything in `shared`.
pub(in crate::dungeon_map) fn noise(a: i64, b: i64, c: i64) -> f32 {
    let mixed =
        (a.wrapping_mul(73_856_093) ^ b.wrapping_mul(19_349_663) ^ c.wrapping_mul(83_492_791))
            .wrapping_mul(2_654_435_761);
    ((mixed >> 16) & 0xffff) as f32 / 32_767.5 - 1.0
}

/// The picture coordinates for a point `across` and at `height` on a face
/// (`across` from the cell's centre), so that it lines up with the next cell.
pub(super) fn face_uv(across: f32, height: f32) -> [f32; 2] {
    [
        (across / CELL_SIZE + 0.5).clamp(0.0, 1.0),
        (1.0 - height / CELL_SIZE).clamp(0.0, 1.0),
    ]
}

/// The picture coordinates for a point on something flat, from the centre.
pub(in crate::dungeon_map) fn flat_uv(east: f32, south: f32) -> [f32; 2] {
    [
        (east / CELL_SIZE + 0.5).clamp(0.0, 1.0),
        (south / CELL_SIZE + 0.5).clamp(0.0, 1.0),
    ]
}

/// The floor a quarter of a wall, door or window takes from the cells round its
/// corner. Liquid is drawn for itself, with this floor round it. Doors, windows
/// and holes have no floor of their own to pass on (a door between wood and
/// grass would give both sides wood), so the cells on the quarter's own side
/// decide.
pub(super) fn floor_near(level: &Level, around: [(i64, i64); 3]) -> Surface {
    around
        .iter()
        .map(|&(x, y)| level.cell(x, y))
        .find(|&cell| {
            cell.is_open() && !cell.looks_liquid() && !cell.is_opening() && cell != Cell::Hole
        })
        .map_or(Surface::Floor, Surface::of_flat)
}
