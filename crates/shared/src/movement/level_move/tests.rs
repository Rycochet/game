use super::*;
use crate::clearance::check_segment;
use crate::dungeon::Cell;

const WALKER: Gait = Gait::UNSKILLED;

fn moved(level: &Level, size: Size, from: (i64, i64), to: Spot) -> Option<LevelMove> {
    move_on_level(level, size, WALKER, Spot::middle_of(from), to)
}

#[test]
fn steps_between_middles_go_by_the_cell_rule() {
    let mut level = Level::test_room();
    let step = |to| moved(&level, Size::Medium, (0, 0), Spot::middle_of(to));
    assert_eq!(
        step((1, 1)),
        Some(LevelMove::Step {
            from: (0, 0),
            to: (1, 1)
        })
    );
    // One cell of water is jumped: two cells in one move.
    level.set(1, 0, Cell::Water);
    let jump = moved(&level, Size::Medium, (0, 0), Spot::middle_of((2, 0)));
    assert!(jump.is_some_and(LevelMove::jumped));
    // Walls touching at a corner: the cell rule lets a Medium creature cut
    // it, but anyone bigger goes by its circle: a Giant has no room between
    // natural walls' feet.
    level.set(3, 2, Cell::Wall);
    level.set(4, 3, Cell::Wall);
    level.set_natural_walls(true);
    let (from, to) = (Spot::middle_of((3, 3)), Spot::middle_of((4, 2)));
    assert!(moved(&level, Size::Medium, (3, 3), to).is_some());
    assert!(moved(&level, Size::Large, (3, 3), to).is_some());
    assert_eq!(check_segment(&level, Size::Giant, from, to), None);
    assert!(moved(&level, Size::Giant, (3, 3), to).is_none());
}

#[test]
fn other_moves_go_by_the_segment_rule() {
    let level = Level::test_room();
    let to = Spot::middle_of((0, 0)).offset(130, -40);
    let segment = moved(&level, Size::Medium, (0, 0), to);
    assert!(matches!(segment, Some(LevelMove::Segment(_))));
    assert!(!segment.unwrap().jumped());
    // Into the wall at (-2, 0) from beside it.
    let into = Spot::middle_of((-2, 0)).offset(100, 0);
    assert_eq!(moved(&level, Size::Medium, (-1, 0), into), None);
}

#[test]
fn moves_are_never_too_long_or_nowhere() {
    let level = Level::test_room();
    let far = Spot::middle_of((3, 0));
    assert_eq!(moved(&level, Size::Medium, (0, 0), far), None);
    let here = Spot::middle_of((0, 0));
    assert_eq!(moved(&level, Size::Medium, (0, 0), here), None);
}

#[test]
fn good_swimmers_swim_water_at_any_angle() {
    let mut skills = crate::skill::Skills::default();
    skills.practise(super::super::MOVEMENT_SKILLS.swimming, 60u32.pow(3));
    let swimmer = Gait::of(&skills);
    assert!(swimmer.swims_deep());
    let mut level = Level::test_room();
    for y in -1..=1 {
        level.set(1, y, Cell::Water);
    }
    // Into the water off the middles: refused walking, swum by a swimmer.
    let from = Spot::middle_of((0, 0)).offset(30, 20);
    let into = Spot::middle_of((1, 0)).offset(-20, 50);
    assert_eq!(
        move_on_level(&level, Size::Medium, WALKER, from, into),
        None
    );
    let swum = move_on_level(&level, Size::Medium, swimmer, from, into);
    assert!(swum.is_some_and(|swum| !swum.jumped()));
    // And on through it, never jumping.
    let out = Spot::middle_of((2, 0)).offset(-60, -30);
    let on = move_on_level(&level, Size::Medium, swimmer, into, out);
    assert!(on.is_some_and(|on| !on.jumped()));
    // Lava is never swum.
    level.set(1, 0, Cell::Lava);
    assert_eq!(
        move_on_level(&level, Size::Medium, swimmer, from, into),
        None
    );
}
