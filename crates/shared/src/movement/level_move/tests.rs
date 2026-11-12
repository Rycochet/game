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
    // it, but anyone bigger goes by its circle, which has no room there.
    level.set(3, 2, Cell::Wall);
    level.set(4, 3, Cell::Wall);
    let (from, to) = (Spot::middle_of((3, 3)), Spot::middle_of((4, 2)));
    assert!(moved(&level, Size::Medium, (3, 3), to).is_some());
    assert_eq!(check_segment(&level, Size::Large, from, to), None);
    assert!(moved(&level, Size::Large, (3, 3), to).is_none());
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
