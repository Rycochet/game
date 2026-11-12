use bevy::math::DVec2;
use shared::clearance::check_segment;
use shared::creature::Size;
use shared::dungeon::{Cell, Level};
use shared::spot::Spot;

use super::*;

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

fn walker(level: &Level) -> Walker<'_> {
    Walker {
        level,
        size: Size::Medium,
        swims: false,
        others: &[],
        squeezes: false,
    }
}

/// Each move starts where the last ended, is straight and legal, and crosses
/// at most one cell's edge.
fn check_chain(level: &Level, start: Spot, moves: &[(Spot, Spot)]) {
    let mut at = start;
    for &(from, to) in moves {
        assert_eq!(from, at);
        assert!(
            check_segment(level, Size::Medium, from, to).is_some(),
            "{from:?} to {to:?}"
        );
        let (a, b) = (from.cell(), to.cell());
        assert!((a.0 - b.0).abs() <= 1 && (a.1 - b.1).abs() <= 1);
        at = to;
    }
}

#[test]
fn walking_east_sends_a_move_a_cell() {
    let level = room(5);
    let start = Spot::middle_of((-3, 0));
    let mut stride = Stride::at(start);
    let mut moves = Vec::new();
    let walked = stride.walk(&walker(&level), DVec2::X, 1200.0, None, &mut moves);
    assert!(!walked.stuck);
    assert_eq!(stride.at, Spot::middle_of((2, 0)));
    moves.extend(stride.finish());
    check_chain(&level, start, &moves);
    assert_eq!(moves.last().map(|m| m.1), Some(Spot::middle_of((2, 0))));
    assert!(moves.len() >= 5);
}

#[test]
fn walking_into_a_wall_at_an_angle_slides_along_it() {
    let level = room(5);
    let start = Spot::middle_of((-3, -4));
    let mut stride = Stride::at(start);
    let mut moves = Vec::new();
    let way = DVec2::new(1.0, -1.0).normalize();
    stride.walk(&walker(&level), way, 1000.0, None, &mut moves);
    moves.extend(stride.finish());
    check_chain(&level, start, &moves);
    // Along the north wall, eastwards, never into it.
    assert!(stride.at.x > start.x + 600, "{:?} {:?}", stride.at, moves);
    assert!(stride.at.y < start.y);
}

#[test]
fn walking_straight_into_a_wall_gets_stuck() {
    let level = room(2);
    let start = Spot::middle_of((0, -2));
    let mut stride = Stride::at(start);
    let mut moves = Vec::new();
    let walked = stride.walk(&walker(&level), -DVec2::Y, 300.0, None, &mut moves);
    assert!(walked.stuck);
    // Not pressed into it: the circle stays clear of the wall's face.
    let face = -2 * 240 - 120;
    assert!(stride.at.y - face >= 80, "{:?}", stride.at);
    moves.extend(stride.finish());
    check_chain(&level, start, &moves);
}

#[test]
fn a_journey_stops_exactly_on_its_point() {
    let level = room(5);
    let start = Spot::middle_of((0, 0));
    let goal = Spot::new(517, 233);
    let way = (DVec2::new(517.0, 233.0)).normalize();
    let mut stride = Stride::at(start);
    let mut moves = Vec::new();
    stride.walk(&walker(&level), way, 5000.0, Some(goal), &mut moves);
    assert_eq!(stride.at, goal);
    moves.extend(stride.finish());
    check_chain(&level, start, &moves);
}

#[test]
fn a_slow_walk_still_gets_somewhere() {
    let level = room(5);
    let start = Spot::middle_of((0, 0));
    let mut stride = Stride::at(start);
    let mut moves = Vec::new();
    let way = DVec2::new(3.0, 1.0).normalize();
    for _ in 0..300 {
        stride.walk(&walker(&level), way, 0.4, None, &mut moves);
    }
    assert!(stride.at.x > 100);
}

#[test]
fn enemies_are_not_walked_into() {
    let level = room(5);
    let start = Spot::middle_of((0, 0));
    let enemy = shared::clearance::Other {
        at: Spot::middle_of((1, 0)),
        size: Size::Medium,
        crowding: shared::clearance::Crowding::Block,
    };
    let walker = Walker {
        level: &level,
        size: Size::Medium,
        swims: false,
        others: &[enemy],
        squeezes: false,
    };
    let mut stride = Stride::at(start);
    let mut moves = Vec::new();
    stride.walk(&walker, DVec2::X, 500.0, None, &mut moves);
    assert_eq!(stride.at.cell(), (0, 0));
    // Up to touching it.
    assert!(stride.at.x >= Spot::middle_of((1, 0)).x - 160 - 40);
}
