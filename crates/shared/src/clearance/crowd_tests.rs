use super::*;
use crate::creature::Size;
use crate::dungeon::{Cell, Level};
use crate::spot::Spot;

fn m(x: i64, y: i64) -> Spot {
    Spot::middle_of((x, y))
}

fn other(at: Spot, size: Size, crowding: Crowding) -> Other {
    Other { at, size, crowding }
}

/// A floor corridor one cell wide, from x = 0 to 4.
fn corridor() -> Level {
    let mut level = Level::new(-1, -1, 7, 3);
    for x in 0..5 {
        level.set(x, 0, Cell::Floor);
    }
    level
}

#[test]
fn players_pass_through_players() {
    let them = [other(m(1, 0), Size::Medium, Crowding::Pass)];
    assert_eq!(
        crowd_per_mille(Size::Medium, m(0, 0), m(2, 0), &them),
        Some(1000)
    );
}

#[test]
fn others_are_squeezed_past_slowly() {
    let them = [other(m(1, 0), Size::Medium, Crowding::Squeeze)];
    // Straight through the middle: as slow as squeezing gets.
    assert_eq!(
        crowd_per_mille(Size::Medium, m(0, 0), m(2, 0), &them),
        Some(4000)
    );
    // Beside, with room to spare, not at all.
    let beside = [other(m(1, 1), Size::Medium, Crowding::Squeeze)];
    assert_eq!(
        crowd_per_mille(Size::Medium, m(0, 0), m(2, 0), &beside),
        Some(1000)
    );
}

#[test]
fn enemies_block() {
    let them = [other(m(1, 0), Size::Medium, Crowding::Block)];
    assert_eq!(crowd_per_mille(Size::Medium, m(0, 0), m(2, 0), &them), None);
    // Up to touching is allowed.
    let near = m(1, 0).offset(-160, 0);
    assert_eq!(
        crowd_per_mille(Size::Medium, m(0, 0), near, &them),
        Some(1000)
    );
    assert_eq!(
        crowd_per_mille(Size::Medium, m(0, 0), near.offset(1, 0), &them),
        None
    );
}

#[test]
fn an_enemy_overlapping_already_can_be_left() {
    let at = m(1, 0);
    let them = [other(at, Size::Medium, Crowding::Block)];
    let start = at.offset(-40, 0);
    assert_eq!(
        crowd_per_mille(Size::Medium, start, start.offset(-100, 0), &them),
        Some(1000)
    );
    assert_eq!(
        crowd_per_mille(Size::Medium, start, start.offset(20, 0), &them),
        None
    );
}

#[test]
fn walls_and_crowds_together() {
    let level = corridor();
    let them = [other(m(2, 0), Size::Medium, Crowding::Squeeze)];
    let passage = check_among(&level, Size::Medium, m(1, 0), m(3, 0), &them).unwrap();
    assert_eq!(passage.squeeze_per_mille, 4000);
    let enemy = [other(m(2, 0), Size::Medium, Crowding::Block)];
    assert_eq!(
        check_among(&level, Size::Medium, m(1, 0), m(3, 0), &enemy),
        None
    );
    // Tiny gets past a Tiny enemy beside it in the corridor.
    let small = [other(m(2, 0).offset(0, -80), Size::Tiny, Crowding::Block)];
    assert!(
        check_among(
            &level,
            Size::Tiny,
            m(1, 0).offset(0, 60),
            m(3, 0).offset(0, 60),
            &small
        )
        .is_some()
    );
}

#[test]
fn crowded_where_circles_overlap() {
    let them = [
        other(m(0, 0), Size::Medium, Crowding::Squeeze),
        other(m(3, 0), Size::Medium, Crowding::Pass),
    ];
    assert!(crowded_at(Size::Medium, m(0, 0).offset(159, 0), &them));
    assert!(!crowded_at(Size::Medium, m(0, 0).offset(160, 0), &them));
    assert!(!crowded_at(Size::Medium, m(3, 0), &them));
}
