use super::*;
use shared::dungeon::Cell;

#[test]
fn mobs_climb_slowly_and_go_on_the_flat_as_ever() {
    let mut level = Level::new(0, 0, 4, 1);
    for x in 0..4 {
        level.set(x, 0, Cell::Floor);
    }
    let (from, to) = (Spot::middle_of((1, 0)), Spot::middle_of((2, 0)));
    assert_eq!(on_ground(&level, None, (from, to), 1050), 1050);
    let hill = Relief::from_fn(level.bounds(), |x, _| x as i32 * 60);
    assert!(on_ground(&level, Some(&hill), (from, to), 1050) > 1050);
    assert!(on_ground(&level, Some(&hill), (to, from), 1050) > 1050);
}
