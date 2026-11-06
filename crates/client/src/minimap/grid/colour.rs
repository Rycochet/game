//! The colour at a point in a cell: ground, hatching and cut corners.

use shared::dungeon::Cell;

use super::{
    CORNER_CUT, CellLook, DOOR, DOOR_HALF_LENGTH, DOOR_HALF_WIDTH, DOOR_LINE, DOOR_OUTLINE, HATCH,
    HATCH_REACH, HATCH_SOLID, HATCH_STROKES, HOLE, HOUSE_DOOR, HOUSE_DOOR_REACH, LINE, Sides,
    WHITE,
};

/// A walkable tile's colour, by what it is: pale, so hatching and doors show.
pub(super) fn ground_colour(cell: Cell) -> [u8; 3] {
    match cell {
        Cell::Ground => [226, 238, 214],
        Cell::CoveredGround => [190, 202, 180],
        Cell::Garden => [200, 230, 190],
        Cell::DirtTrack => [222, 205, 180],
        Cell::GravelTrack => [218, 212, 198],
        Cell::WoodenBoards => [232, 212, 178],
        Cell::Cobblestone => [214, 210, 206],
        Cell::Flagstones => [226, 220, 210],
        Cell::DressedStone => [222, 226, 232],
        Cell::Water => [150, 195, 240],
        Cell::Shallows => [180, 214, 244],
        Cell::Lava => [240, 140, 120],
        Cell::Hole => HOLE,
        Cell::Field => [214, 222, 160],
        Cell::Fence => [150, 110, 70],
        Cell::FieldGate => [226, 238, 214],
        Cell::HerbBed => [205, 222, 180],
        Cell::Grass => [176, 214, 150],
        Cell::Tree => [96, 150, 84],
        _ => WHITE,
    }
}

/// The colour at a point in a cell (`u` east and `v` south of its centre, in
/// cells), if any. `line` is half a grid line's width.
pub fn colour_at(look: CellLook, u: f64, v: f64, line: f64) -> Option<[u8; 3]> {
    let floor = |colour: [u8; 3]| {
        // (A hair wider than a pixel: exactly a pixel can fall between samples.)
        if u.abs().max(v.abs()) > 0.5 - line * 1.01 {
            LINE
        } else {
            colour
        }
    };
    let along_across = |east_west: bool| {
        if east_west {
            (u.abs(), v.abs())
        } else {
            (v.abs(), u.abs())
        }
    };
    match look {
        CellLook::Empty => None,
        CellLook::Floor(colour) => Some(floor(colour)),
        CellLook::Wall(sides) => {
            cut_colour(sides, u, v).or_else(|| hatched(sides, u, v).then_some(HATCH))
        }
        CellLook::Sign(_) => Some(floor(WHITE)),
        CellLook::HouseDoor { east_west } => {
            let (along, across) = along_across(east_west);
            Some(if along + across <= HOUSE_DOOR_REACH {
                HOUSE_DOOR
            } else if across <= DOOR_LINE {
                HATCH
            } else {
                floor(WHITE)
            })
        }
        CellLook::Window { east_west, sides } => {
            let (_, across) = along_across(east_west);
            let signed = if east_west { v } else { u };
            Some(if across < DOOR_HALF_WIDTH - DOOR_OUTLINE {
                floor(WHITE)
            } else if across <= DOOR_HALF_WIDTH {
                HATCH
            } else {
                floor(sides[usize::from(signed > 0.0)])
            })
        }
        CellLook::Door { east_west } | CellLook::SecretDoor { east_west } => {
            let (along, across) = along_across(east_west);
            let in_rectangle = along <= DOOR_HALF_LENGTH && across <= DOOR_HALF_WIDTH;
            Some(if in_rectangle {
                let inner = along <= DOOR_HALF_LENGTH - DOOR_OUTLINE
                    && across <= DOOR_HALF_WIDTH - DOOR_OUTLINE;
                if inner { DOOR } else { HATCH }
            } else if across <= DOOR_LINE {
                HATCH
            } else {
                floor(WHITE)
            })
        }
    }
}

/// The colour of a corner cut away from a wall, if the point is in one.
pub(super) fn cut_colour(sides: Sides, u: f64, v: f64) -> Option<[u8; 3]> {
    let corner = match (u > 0.0, v > 0.0) {
        (true, false) => 0,
        (true, true) => 1,
        (false, true) => 2,
        (false, false) => 3,
    };
    let from_corner = (0.5 - u.abs()) + (0.5 - v.abs());
    sides.cut[corner].filter(|_| from_corner < CORNER_CUT)
}

/// Whether a point in a wall cell is on a stroke of the hatching along a side.
pub(super) fn hatched(sides: Sides, u: f64, v: f64) -> bool {
    let near = 0.5 - HATCH_REACH;
    let by_edge = (sides.north && v < -near)
        || (sides.south && v > near)
        || (sides.west && u < -near)
        || (sides.east && u > near);
    by_edge && ((u + v) * HATCH_STROKES).rem_euclid(1.0) < HATCH_SOLID
}
