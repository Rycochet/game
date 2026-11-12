//! Routes by size, straightened to any angle.

use super::*;
use crate::clearance::{RoomMap, check_segment};
use crate::creature::Size;
use crate::door::{Side, Swing};
use crate::dungeon::{Cell, Level};
use crate::spot::Spot;

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

fn route(level: &Level, size: Size, from: Spot, to: Spot) -> SizedPath {
    let path = find_sized_path(level, &RoomMap::new(level, size), from, to, None, |_, _| {
        true
    });
    for pair in path.spots.windows(2) {
        assert!(
            check_segment(level, size, pair[0], pair[1]).is_some(),
            "{pair:?}"
        );
    }
    path
}

/// A wall north–south at `x = 0`, with an open door at `(0, 3)`.
fn walled() -> Level {
    let mut level = room(4);
    for y in -4..=4 {
        level.set(0, y, Cell::Wall);
    }
    level.set(0, 3, Cell::Door);
    level.set_swing(0, 3, Swing::Open(Side::East));
    level
}

#[test]
fn across_open_floor_in_one_line() {
    let level = room(5);
    let (from, to) = (m(-3, -2).offset(17, -40), m(4, 1).offset(-33, 90));
    let path = route(&level, Size::Medium, from, to);
    assert!(path.reached);
    assert_eq!(path.spots, vec![from, to]);
    assert_eq!(path.cost, from.distance(to));
}

#[test]
fn through_a_door_by_size() {
    let level = walled();
    let (from, to) = (m(-3, -3), m(3, -3));
    let medium = route(&level, Size::Medium, from, to);
    assert!(medium.reached);
    assert!(medium.spots.len() >= 4, "{:?}", medium.spots);
    let large = route(&level, Size::Large, from, to);
    assert!(large.reached);
    assert!(large.cost > medium.cost, "Large squeezes through the door");
    let giant = route(&level, Size::Giant, m(-2, 0), m(2, 0));
    assert!(!giant.reached, "Giant never fits through a door");
    // A closed door stops everyone.
    let mut shut = walled();
    shut.set_swing(0, 3, Swing::Closed);
    assert!(!route(&shut, Size::Tiny, from, to).reached);
}

#[test]
fn routes_jump_a_stream() {
    let mut level = room(3);
    for y in -3..=3 {
        level.set(0, y, Cell::Water);
    }
    let path = route(&level, Size::Medium, m(-2, -1), m(2, 1));
    assert!(path.reached);
    let jumps = path
        .spots
        .windows(2)
        .filter(|pair| {
            check_segment(&level, Size::Medium, pair[0], pair[1])
                .unwrap()
                .jumped
                .is_some()
        })
        .count();
    assert_eq!(jumps, 1);
}

#[test]
fn routes_are_the_same_every_time() {
    let level = walled();
    let a = route(&level, Size::Small, m(-4, 4), m(4, -4));
    let b = route(&level, Size::Small, m(-4, 4), m(4, -4));
    assert_eq!(a, b);
    assert!(a.reached);
}

#[test]
fn nowhere_to_start() {
    let level = walled();
    let path = route(&level, Size::Medium, m(0, 0), m(3, 0));
    assert_eq!(path, SizedPath::default());
}

#[test]
fn distance_maps_by_size() {
    // Two rooms, five cells square, with an open doorway between them.
    let mut level = Level::new(-6, -3, 13, 7);
    for y in -2..=2 {
        for x in -5..=5 {
            level.set(x, y, if x == 0 { Cell::Wall } else { Cell::Floor });
        }
    }
    level.set(0, 0, Cell::Door);
    level.set_swing(0, 0, Swing::Open(Side::East));
    let goal = (3, 0);
    let medium = DistanceMap::to(&level, goal);
    let large = DistanceMap::sized(&level, Size::Large, goal);
    let giant = DistanceMap::sized(&level, Size::Giant, goal);
    let from = (-3, 0);
    // Large squeezes through, so it takes longer; a Giant never does.
    assert!(large.cost(from).unwrap() > medium.cost(from).unwrap());
    assert_eq!(giant.cost(from), None);
    assert!(giant.cost((4, 0)).is_some());
    assert!(large.towards(&level, from, |_| true).is_some());
}
