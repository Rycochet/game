use super::*;
use crate::creature::Size;
use crate::door::{Side, Swing};
use crate::dungeon::{Cell, Level};
use crate::spot::Spot;

/// A floor room from `(-r, -r)` to `(r, r)`, walled round.
fn room(r: i64) -> Level {
    let mut level = Level::new(-r - 1, -r - 1, 2 * r + 3, 2 * r + 3);
    for y in -r..=r {
        for x in -r..=r {
            level.set(x, y, Cell::Floor);
        }
    }
    level
}

fn m(x: i64, y: i64) -> Spot {
    Spot::middle_of((x, y))
}

fn squeeze(level: &Level, size: Size, from: Spot, to: Spot) -> Option<u32> {
    check_segment(level, size, from, to).map(|p| p.squeeze_per_mille)
}

#[test]
fn radii_by_size() {
    let radii = Size::ALL.map(Size::radius);
    assert_eq!(radii, [20, 60, 80, 120, 180, 240]);
    // Medium's base is 2/3 of a cell across, as tokens were drawn.
    assert_eq!(Size::Medium.radius() * 2 * 3, 2 * 240);
}

#[test]
fn squeezing_by_the_owners_rule() {
    assert_eq!(squeeze_per_mille(80, 80), Some(1000));
    assert_eq!(squeeze_per_mille(80, 500), Some(1000));
    assert_eq!(squeeze_per_mille(80, 40), Some(MOST_SQUEEZE_PER_MILLE));
    assert_eq!(squeeze_per_mille(80, 39), None);
    assert_eq!(squeeze_per_mille(120, 96), Some(1562));
}

#[test]
fn open_floor_is_free_for_everyone_who_fits() {
    let level = room(6);
    for size in [Size::Tiny, Size::Small, Size::Medium, Size::Large] {
        let passage = check_segment(&level, size, m(-2, -1), m(3, 2)).unwrap();
        assert_eq!(
            passage,
            Passage {
                squeeze_per_mille: 1000,
                jumped: None
            }
        );
    }
    // Walls round the room: no footing beyond it, and none into it.
    assert!(!segment_is_legal(&level, Size::Medium, m(0, 0), m(7, 0)));
}

#[test]
fn a_corridor_one_cell_wide() {
    let mut level = Level::new(-8, -1, 17, 3);
    for x in -7..=7 {
        level.set(x, 0, Cell::Floor);
    }
    let along = |size| squeeze(&level, size, m(-3, 0), m(3, 0));
    assert_eq!(along(Size::Medium), Some(1000));
    assert_eq!(along(Size::Large), Some(1000));
    assert_eq!(along(Size::Huge), Some(2250));
    assert_eq!(along(Size::Giant), Some(4000));
    assert_eq!(
        segment_cost(&level, Size::Huge, m(-3, 0), m(3, 0)),
        Some(3240)
    );
}

/// A wall running north–south at `x = 0` across a room, a door in it at
/// `(0, 0)`.
fn doorway(swing: Swing) -> Level {
    let mut level = room(3);
    for y in -3..=3 {
        level.set(0, y, Cell::Wall);
    }
    level.set(0, 0, Cell::Door);
    level.set_swing(0, 0, swing);
    level
}

#[test]
fn through_a_door_by_size() {
    let level = doorway(Swing::Open(Side::East));
    let through = |size| squeeze(&level, size, m(-1, 0), m(1, 0));
    // The doorway is 192 units between its posts.
    assert_eq!(through(Size::Tiny), Some(1000));
    assert_eq!(through(Size::Small), Some(1000));
    assert_eq!(through(Size::Medium), Some(1000));
    // Large gets through slowly; Huge needs 0.75 cell and the doorway is 0.8.
    assert_eq!(through(Size::Large), Some(1562));
    assert_eq!(through(Size::Huge), Some(3515));
    assert_eq!(through(Size::Giant), None);
    // Broken doors leave the same frame; closed ones block.
    let broken = doorway(Swing::Broken(Side::West));
    assert_eq!(squeeze(&broken, Size::Large, m(-1, 0), m(1, 0)), Some(1562));
    let closed = doorway(Swing::Closed);
    assert_eq!(squeeze(&closed, Size::Tiny, m(-1, 0), m(1, 0)), None);
    // Into the post beside the doorway.
    assert!(!segment_is_legal(&level, Size::Medium, m(-1, 1), m(1, -1)));
}

#[test]
fn between_walls_touching_at_a_corner() {
    let mut level = room(3);
    level.set(1, 0, Cell::Wall);
    level.set(0, 1, Cell::Wall);
    let across = |size| squeeze(&level, size, m(0, 0), m(1, 1));
    assert_eq!(across(Size::Tiny), Some(1000));
    assert_eq!(across(Size::Small), Some(2040));
    assert_eq!(across(Size::Medium), Some(3628));
    assert_eq!(across(Size::Large), None);
}

#[test]
fn round_a_tree_trunk() {
    let mut level = room(3);
    level.set(1, 0, Cell::Tree);
    assert!(!segment_is_legal(&level, Size::Tiny, m(0, 0), m(2, 0)));
    // Past it, 110 units to one side: 80 units clear of the trunk.
    let (a, b) = (m(0, 0).offset(0, 110), m(2, 0).offset(0, 110));
    assert_eq!(squeeze(&level, Size::Medium, a, b), Some(1000));
    assert_eq!(squeeze(&level, Size::Large, a, b), Some(2250));
    // A tree's cell has footing beside the trunk.
    assert!(segment_is_legal(
        &level,
        Size::Tiny,
        m(0, 0),
        m(1, 0).offset(0, -70)
    ));
}

#[test]
fn jumps_go_over_one_cell() {
    let mut level = room(4);
    level.set(1, 0, Cell::Water);
    let jump = check_segment(&level, Size::Medium, m(0, 0), m(2, 0)).unwrap();
    assert_eq!(jump.jumped, Some((1, 0)));
    assert!(!segment_is_legal(&level, Size::Medium, m(0, 0), m(1, 0)));
    // Diagonally, two cells: as far as a jump goes.
    level.set(1, 1, Cell::Hole);
    assert!(segment_is_legal(&level, Size::Medium, m(0, 0), m(2, 2)));
    // Not over two cells, and not further than the reach.
    level.set(2, 0, Cell::Lava);
    assert!(!segment_is_legal(&level, Size::Medium, m(0, 0), m(3, 0)));
    let mut wide = room(4);
    wide.set(0, 0, Cell::Water);
    assert!(segment_is_legal(
        &wide,
        Size::Tiny,
        m(-1, 0),
        m(1, 0).offset(100, 0)
    ));
    assert!(!segment_is_legal(&wide, Size::Tiny, m(-2, 0), m(2, 0)));
    // Along the water's edge is a walk.
    let edge = check_segment(&level, Size::Small, m(0, -1), m(3, -1)).unwrap();
    assert_eq!(edge.jumped, None);
}

#[test]
fn fences_are_thin_and_jumped() {
    let mut level = room(4);
    for y in -4..=4 {
        level.set(1, y, Cell::Fence);
    }
    level.set(1, 3, Cell::FieldGate);
    let jump = check_segment(&level, Size::Medium, m(0, 0), m(2, 0)).unwrap();
    assert_eq!(jump.jumped, Some((1, 0)));
    // Through the gate is a walk.
    let gate = check_segment(&level, Size::Medium, m(0, 3), m(2, 3)).unwrap();
    assert_eq!(
        gate,
        Passage {
            squeeze_per_mille: 1000,
            jumped: None
        }
    );
    // Standing in a fence's cell, beside the bar.
    assert!(segment_is_legal(
        &level,
        Size::Small,
        m(0, 0),
        m(1, 0).offset(-90, 0)
    ));
    // Walking along close beside it squeezes.
    let (a, b) = (m(1, -2).offset(-60, 0), m(1, 2).offset(-60, 0));
    assert_eq!(
        squeeze(&level, Size::Medium, a, b),
        Some(1000 * 6400 / (48 * 48))
    );
}

#[test]
fn sliding_along_a_wall() {
    let level = room(2);
    // Up and to the east into the north wall: slides east along it, one
    // radius off it.
    let end = slide(&level, Size::Medium, m(0, 1), m(2, -3));
    assert_eq!(end.x, m(2, 0).x);
    assert!((-522..=-518).contains(&end.y), "{end:?}");
    // A legal move goes all the way, squeezing if it must.
    assert_eq!(slide(&level, Size::Medium, m(0, 0), m(1, 1)), m(1, 1));
    // No room where it starts: it stays.
    let door = doorway(Swing::Open(Side::North));
    assert_eq!(slide(&door, Size::Giant, m(0, 0), m(1, 0)), m(0, 0));
    // Never jumps.
    let mut pool = room(3);
    pool.set(1, 0, Cell::Water);
    let end = slide(&pool, Size::Medium, m(0, 0), m(2, 0));
    assert!(end.x < m(1, 0).x - 119, "{end:?}");
}

#[test]
fn room_maps_by_size() {
    let level = doorway(Swing::Open(Side::North));
    let medium = RoomMap::new(&level, Size::Medium);
    assert_eq!(medium.squeeze_at(0, 0), Some(1000));
    assert_eq!(medium.squeeze_at(-1, 0), Some(1000));
    assert!(!medium.has_room(0, 1));
    assert!(!medium.has_room(9, 9));
    let huge = RoomMap::new(&level, Size::Huge);
    assert_eq!(huge.squeeze_at(0, 0), Some(3515));
    assert!(!RoomMap::new(&level, Size::Giant).has_room(0, 0));
}

#[test]
fn blockers_of_cells() {
    let level = doorway(Swing::Open(Side::North));
    let mut out = Vec::new();
    cell_blockers(&level, 0, 0, &mut out);
    assert_eq!(out.len(), 2, "a door's frame: two posts");
    out.clear();
    // The wall's end at the room's edge has its free corners notched.
    cell_blockers(&level, 0, 3, &mut out);
    assert_eq!(out.len(), 1, "joined below by the level's edge");
    let mut lone = room(2);
    lone.set(0, 0, Cell::Wall);
    out.clear();
    cell_blockers(&lone, 0, 0, &mut out);
    assert_eq!(out.len(), 3, "a column: every corner notched");
}
