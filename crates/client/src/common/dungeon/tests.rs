use shared::door::{Side, Swing};
use shared::dungeon::{Cell, Level, Sight};

use super::CurrentDungeon;

/// A dungeon whose level has a closed door at (5, 5), a locked one at (6, 5)
/// and an open one at (7, 5), all sent by the server.
fn with_doors() -> CurrentDungeon {
    let mut dungeon = CurrentDungeon::default();
    dungeon.level = Level::new(0, 0, 10, 10);
    dungeon.known = Sight::new(&dungeon.level);
    dungeon.remembered = Sight::new(&dungeon.level);
    for x in 5..=7 {
        dungeon.level.set(x, 5, Cell::Door);
        dungeon.known.insert(x, 5);
    }
    dungeon.level.set_swing(5, 5, Swing::Closed);
    dungeon.level.set_locked(6, 5, true);
    dungeon.level.set_swing(7, 5, Swing::Open(Side::North));
    dungeon
}

#[test]
fn doors_hang_the_same_after_the_window_moves() {
    let mut dungeon = with_doors();
    dungeon.move_window((2, 2, 10, 10));
    let level = &dungeon.level;
    assert!(level.is_shut(5, 5), "a closed door stays closed");
    assert!(
        level.is_shut(6, 5) && level.is_locked(6, 5),
        "and a locked one locked"
    );
    assert_eq!(level.swing(7, 5), Some(Swing::Open(Side::North)));
}

#[test]
fn doors_left_outside_the_window_are_dropped() {
    let mut dungeon = with_doors();
    dungeon.move_window((6, 0, 10, 10));
    let level = &dungeon.level;
    assert_eq!(level.swing(5, 5), None);
    assert!(level.is_shut(6, 5) && level.is_locked(6, 5));
}
