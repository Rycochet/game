use super::*;
use crate::dungeon::Cell;

/// Open ground, three blocks wide, with a wall across part of it.
fn land() -> Level {
    let mut level = Level::new(0, 0, 192, 192);
    for y in 0..192 {
        for x in 0..192 {
            let wall = x == 100 && (60..140).contains(&y);
            level.set(x, y, if wall { Cell::Wall } else { Cell::Grass });
        }
    }
    level
}

#[test]
fn trails_are_the_same_every_time_and_differ_between_mobs() {
    let level = land();
    let seed = Seed::new(3);
    let trail = wild_trail(seed, 9, (96, 96), &level);
    assert_eq!(trail, wild_trail(seed, 9, (96, 96), &level));
    assert_ne!(trail, wild_trail(seed, 10, (96, 96), &level));
}

#[test]
fn trails_start_at_home_and_go_by_legal_steps() {
    let level = land();
    for number in 0..40 {
        let trail = wild_trail(Seed::new(5), number, (96, 96), &level);
        assert_eq!(trail[0], (96, 96));
        assert!(
            trail.len() as i64 > TRAIL_SHORTEST / 2,
            "{number}: too short"
        );
        assert!(trail.len() as i64 <= TRAIL_LONGEST + 1);
        for pair in trail.windows(2) {
            let (dx, dy) = (pair[1].0 - pair[0].0, pair[1].1 - pair[0].1);
            let step = Step::new(dx, dy).expect("one cell at a time on open ground");
            assert_eq!(step_on_level(&level, pair[0], step), Some(pair[1]));
        }
    }
}

#[test]
fn trails_stop_at_the_level_edge() {
    let level = land();
    for number in 0..40 {
        let trail = wild_trail(Seed::new(8), number, (2, 2), &level);
        assert!(trail.iter().all(|&(x, y)| level.contains(x, y)));
    }
}
