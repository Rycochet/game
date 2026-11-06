//! What is round a liquid cell: its outline in each quarter, surfaces and light.

use super::LooksLiquid;
use bevy::prelude::*;
use shared::dungeon::{Cell, Level};

use super::super::mesh::Surface;
use super::super::{CELL_SIZE, mesh::LightGrid};
use super::Outline;

/// Whether liquid carries on past this cell: it is liquid, or a wall.
pub(super) fn wet(cell: Cell) -> bool {
    cell.looks_liquid() || cell.is_wall()
}

/// The three other cells round the corner of `(x, y)` in a quarter, beside it,
/// ahead of it and across the corner.
pub(super) fn around((x, y): (i64, i64), (sx, sz): (f32, f32)) -> [(i64, i64); 3] {
    let (dx, dy) = (sx as i64, sz as i64);
    [(x + dx, y), (x, y + dy), (x + dx, y + dy)]
}

/// The liquid in a quarter of a cell, if any: a liquid cell always has some,
/// and a wall where it meets any. Beside a wall, liquid stays in the same
/// shape: the wall only counts as liquid if the liquid comes round to it.
pub fn outline_at(level: &Level, (x, y): (i64, i64), quarter: (f32, f32)) -> Option<Outline> {
    let cell = level.cell(x, y);
    let [a, b, c] = around((x, y), quarter).map(|(x, y)| level.cell(x, y));
    if cell.looks_liquid() {
        return Some(Outline::of(wet(a), wet(b), wet(c)));
    }
    if !cell.is_wall() || ![a, b, c].iter().any(|cell| cell.looks_liquid()) {
        return None;
    }
    let joins =
        |near: Cell, other: Cell| near.looks_liquid() || (near.is_wall() && wet(other) && wet(c));
    let (beside, ahead) = (joins(a, b), joins(b, a));
    if !beside && !ahead {
        return None;
    }
    Some(Outline::of(beside, ahead, wet(c)))
}

/// The liquid and the floor round the corners of `(x, y)`, for its sides and
/// bed: the first of each found, nearest first.
pub fn surfaces_near(level: &Level, (x, y): (i64, i64)) -> (Surface, Surface) {
    const NEAR: [(i64, i64); 9] = [
        (0, 0),
        (0, -1),
        (1, 0),
        (0, 1),
        (-1, 0),
        (1, -1),
        (1, 1),
        (-1, 1),
        (-1, -1),
    ];
    let cells = || NEAR.iter().map(|&(dx, dy)| level.cell(x + dx, y + dy));
    let liquid = cells()
        .find(|cell| cell.looks_liquid())
        .map_or(Surface::Water, Surface::of_flat);
    let floor = cells()
        .find(|&cell| cell.is_open() && !cell.looks_liquid())
        .map_or(Surface::Floor, Surface::of_flat);
    (liquid, floor)
}

/// The light on a liquid cell at a point: between its corners, like floors.
pub(super) fn cell_light(
    level: &Level,
    light: &LightGrid,
    (x, y): (i64, i64),
    point: Vec3,
) -> Vec3 {
    let east = (point.x / CELL_SIZE - x as f32 + 0.5).clamp(0.0, 1.0);
    let south = (point.z / CELL_SIZE - y as f32 + 0.5).clamp(0.0, 1.0);
    let here = light.at(x, y);
    let corner = |cx, cy| light.corner(level, cx, cy).unwrap_or(here);
    let north = corner(x, y).lerp(corner(x + 1, y), east);
    let below = corner(x, y + 1).lerp(corner(x + 1, y + 1), east);
    north.lerp(below, south)
}
