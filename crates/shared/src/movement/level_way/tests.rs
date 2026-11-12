use super::*;
use crate::dungeon::Cell;
use crate::energy::{level_move_energy, site_move_cost};
use crate::movement::{MOVEMENT_SKILLS, Terrain};
use crate::skill::Skills;

fn floor(width: i64) -> Level {
    let mut level = Level::new(0, 0, width, 3);
    for y in 0..3 {
        for x in 0..width {
            level.set(x, y, Cell::Floor);
        }
    }
    level
}

fn way<'a>(level: &'a Level, relief: Option<&'a Relief>) -> LevelWay<'a> {
    LevelWay {
        level,
        relief,
        terrain: None,
        gait: Gait::UNSKILLED,
    }
}

fn climber(percent: u32) -> Gait {
    let mut skills = Skills::default();
    skills.practise(MOVEMENT_SKILLS.climbing, percent.pow(3));
    Gait::of(&skills)
}

#[test]
fn flat_ground_costs_its_length_as_ever() {
    let level = floor(8);
    let flat = Relief::flat(level.bounds());
    for relief in [None, Some(&flat)] {
        let way = way(&level, relief);
        for (from, to) in [
            (Spot::middle_of((0, 1)), Spot::middle_of((1, 1))),
            (Spot::middle_of((0, 0)), Spot::middle_of((1, 1))),
            (Spot::new(13, 250), Spot::new(1201, 300)),
        ] {
            let cost = way.cost(from, to, false);
            assert_eq!(cost, from.distance(to));
            assert_eq!(level_move_energy(cost), site_move_cost(from, to));
            assert_eq!(way.climb(from, to), 0);
        }
    }
}

#[test]
fn slopes_cost_extra_less_by_climbing() {
    let level = floor(8);
    // Rising a 16th of an elevation a cell eastwards.
    let slope = Relief::from_fn(level.bounds(), |x, _| x as i32 * UNITS_PER_16TH);
    let way = way(&level, Some(&slope));
    let (from, to) = (Spot::middle_of((1, 1)), Spot::middle_of((2, 1)));
    assert_eq!(way.climb(from, to), UNITS_PER_16TH as u32);
    assert_eq!(way.climb(to, from), UNITS_PER_16TH as u32);
    assert_eq!(way.steepness(from, to), 1);
    // A 16th climbed costs SLOPE_COST on top of a straight step, as in the
    // wilderness before (`wild_step_cost`).
    let energy = level_move_energy(way.cost(from, to, false));
    let flat = site_move_cost(from, to);
    let expected = (STRAIGHT_COST + SLOPE_COST) * crate::energy::SITE_STEP_SCALE;
    assert!(energy.abs_diff(expected) <= crate::energy::SITE_STEP_SCALE);
    assert!(energy > flat);
    // Along the slope, nothing is climbed.
    let across = (Spot::middle_of((1, 0)), Spot::middle_of((1, 2)));
    assert_eq!(way.climb(across.0, across.1), 0);
    // A good climber hardly notices it.
    let good = LevelWay {
        gait: climber(100),
        ..way
    };
    assert!(good.cost(from, to, false) < way.cost(from, to, false));
    assert!(good.cost(from, to, false) > from.distance(to));
}

#[test]
fn a_hump_between_two_spots_counts() {
    let level = floor(8);
    // A ridge at x = 2's corners.
    let ridge = Relief::from_fn(level.bounds(), |x, _| if x == 2 { 120 } else { 0 });
    let way = way(&level, Some(&ridge));
    let (from, to) = (Spot::middle_of((0, 1)), Spot::middle_of((3, 1)));
    // Up and down again: both count.
    assert_eq!(way.climb(from, to), 240);
}

#[test]
fn wading_and_terrain_slow_the_pieces_in_them() {
    let mut level = floor(4);
    level.set(2, 1, Cell::Shallows);
    let dry = way(&level, None);
    let (from, to) = (Spot::middle_of((0, 1)), Spot::middle_of((3, 1)));
    assert!(dry.cost(from, to, false) > from.distance(to));
    // A jump over the shallows lands dry, and is taken whole.
    let (start, end) = (Spot::middle_of((1, 1)), Spot::middle_of((3, 1)));
    assert!(dry.cost(start, end, true) <= start.distance(end));
    // Terrain slows open ground only.
    level.set(0, 1, Cell::Ground);
    let wild = LevelWay {
        terrain: Some(Terrain::Desert),
        ..way(&level, None)
    };
    let near = (Spot::middle_of((0, 1)), Spot::new(100, 240));
    assert_eq!(
        wild.cost(near.0, near.1, false),
        2 * near.0.distance(near.1)
    );
}
