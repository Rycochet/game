use shared::dungeon::{Cell, Level};

use super::super::mesh::Surface;
use super::*;

/// Floor to the north, ground to the south, and a wall line between them
/// along `y` = 1 with `middle` at `x` = 1.
fn wall_line(middle: Cell) -> Level {
    let mut level = Level::new(0, 0, 3, 3);
    for x in 0..3 {
        level.set(x, 0, Cell::Floor);
        level.set(x, 1, Cell::Wall);
        level.set(x, 2, Cell::Ground);
    }
    level.set(1, 1, middle);
    level
}

/// The cells round the corner of a quarter of the cell at `(x, y)`.
fn around((x, y): (i64, i64), (dx, dy): (i64, i64)) -> [(i64, i64); 3] {
    [(x + dx, y), (x, y + dy), (x + dx, y + dy)]
}

#[test]
fn a_wall_beside_an_opening_takes_the_floor_on_its_own_side() {
    for middle in [Cell::Door, Cell::Window] {
        let level = wall_line(middle);
        let north = floor_near(&level, around((0, 1), (1, -1)));
        let south = floor_near(&level, around((0, 1), (1, 1)));
        assert_eq!(north, Surface::Floor, "{middle:?}");
        assert_eq!(south, Surface::Ground, "{middle:?}");
    }
}

#[test]
fn an_opening_beside_an_opening_takes_the_floor_on_its_own_side() {
    let mut level = wall_line(Cell::Door);
    level.set(0, 1, Cell::Window);
    assert_eq!(floor_near(&level, around((0, 1), (1, 1))), Surface::Ground);
    assert_eq!(floor_near(&level, around((1, 1), (-1, 1))), Surface::Ground);
}

#[test]
fn a_wall_beside_a_hole_takes_the_floor_on_its_own_side() {
    let level = wall_line(Cell::Hole);
    assert_eq!(floor_near(&level, around((0, 1), (1, 1))), Surface::Ground);
}

#[test]
fn only_openings_round_a_corner_give_floor() {
    let mut level = Level::new(0, 0, 2, 2);
    level.set(0, 0, Cell::Wall);
    level.set(1, 0, Cell::Door);
    level.set(0, 1, Cell::Wall);
    level.set(1, 1, Cell::Wall);
    assert_eq!(floor_near(&level, around((0, 0), (1, 1))), Surface::Floor);
}
