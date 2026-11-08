use super::can_leave;
use crate::dungeon::{Cell, Level};
use crate::poi::PoiKind;

/// A 9 × 9 level of floor with outside ground in the middle and along the
/// west edge.
fn level() -> Level {
    let mut level = Level::new(0, 0, 9, 9);
    for y in 0..9 {
        for x in 0..9 {
            level.set(x, y, Cell::Floor);
        }
        level.set(0, y, Cell::Ground);
    }
    level.set(4, 4, Cell::Ground);
    level.set(4, 5, Cell::Cobblestone);
    level
}

#[test]
fn town_leaves_from_any_outside_cell() {
    let level = level();
    assert!(can_leave(PoiKind::Village, 0, &level, 4, 4));
    assert!(can_leave(PoiKind::Village, 0, &level, 4, 5));
    assert!(can_leave(PoiKind::Village, 0, &level, 0, 3));
    assert!(!can_leave(PoiKind::Village, 0, &level, 5, 5));
}

#[test]
fn cave_leaves_only_from_its_edge() {
    let level = level();
    assert!(can_leave(PoiKind::RockCave, 0, &level, 0, 3));
    assert!(!can_leave(PoiKind::RockCave, 0, &level, 4, 4));
}

#[test]
fn only_from_the_level_entered_from_the_world_map() {
    let level = level();
    assert!(!can_leave(PoiKind::Village, 1, &level, 4, 4));
    assert!(!can_leave(PoiKind::RockCave, 1, &level, 0, 3));
}
