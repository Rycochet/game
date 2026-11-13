use shared::door::{Side, Swing};
use shared::dungeon::{Cell, Level};
use shared::facing::Heading;
use shared::protocol::DoorAsk;

use super::door_ask;

/// Floor all round, the player at (5, 5), and doors as given.
fn level(doors: &[((i64, i64), Swing)]) -> Level {
    let mut level = Level::new(0, 0, 11, 11);
    for y in 0..11 {
        for x in 0..11 {
            level.set(x, y, Cell::Floor);
        }
    }
    for &((x, y), swing) in doors {
        level.set(x, y, Cell::Door);
        level.set_swing(x, y, swing);
    }
    level
}

#[test]
fn an_open_door_beside_is_closed_first() {
    let level = level(&[((6, 5), Swing::Closed), ((4, 5), Swing::Open(Side::West))]);
    assert_eq!(
        door_ask(&level, (5, 5), Heading::EAST),
        Some(DoorAsk::CloseNear)
    );
}

#[test]
fn the_closed_door_faced_opens() {
    let level = level(&[((6, 5), Swing::Closed), ((5, 4), Swing::Closed)]);
    assert_eq!(
        door_ask(&level, (5, 5), Heading::NORTH),
        Some(DoorAsk::Open { x: 5, y: 4 })
    );
    assert_eq!(
        door_ask(&level, (5, 5), Heading::EAST),
        Some(DoorAsk::Open { x: 6, y: 5 })
    );
}

#[test]
fn a_door_behind_opens_when_it_is_the_only_one() {
    let level = level(&[((5, 6), Swing::Closed)]);
    assert_eq!(
        door_ask(&level, (5, 5), Heading::NORTH),
        Some(DoorAsk::Open { x: 5, y: 6 })
    );
}

#[test]
fn no_door_near_asks_nothing() {
    let level = level(&[
        ((8, 5), Swing::Closed),
        ((5, 6), Swing::Broken(Side::North)),
    ]);
    assert_eq!(door_ask(&level, (5, 5), Heading::EAST), None);
}
