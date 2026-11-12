//! Sized routes weigh slopes and let good swimmers swim.

use super::*;
use crate::clearance::{RoomMap, check_segment_by};
use crate::creature::Size;
use crate::dungeon::{Cell, Level};
use crate::movement::{Gait, LevelWay, MOVEMENT_SKILLS};
use crate::relief::Relief;
use crate::skill::Skills;
use crate::spot::Spot;

fn field(r: i64) -> Level {
    let mut level = Level::new(-r - 1, -r - 1, 2 * r + 3, 2 * r + 3);
    for y in -r..=r {
        for x in -r..=r {
            level.set(x, y, Cell::Ground);
        }
    }
    level
}

fn m(x: i64, y: i64) -> Spot {
    Spot::middle_of((x, y))
}

fn route(way: &LevelWay, from: Spot, to: Spot) -> SizedPath {
    let rooms = RoomMap::new_by(way.level, Size::Medium, way.gait.swims_deep());
    let path = find_sized_path(way, &rooms, from, to, None, |_, _| true);
    for pair in path.spots.windows(2) {
        let swims = way.gait.swims_deep();
        let legal = check_segment_by(way.level, Size::Medium, swims, pair[0], pair[1]);
        assert!(legal.is_some(), "{pair:?}");
    }
    path
}

fn swimmer(percent: u32) -> Gait {
    let mut skills = Skills::default();
    skills.practise(MOVEMENT_SKILLS.swimming, percent.pow(3));
    Gait::of(&skills)
}

/// The furthest a route's spots go from the line `y = 0`, in units.
fn widest(path: &SizedPath) -> i64 {
    path.spots
        .iter()
        .map(|spot| spot.y.abs())
        .max()
        .unwrap_or(0)
}

#[test]
fn routes_go_round_a_steep_hill() {
    let level = field(8);
    // A steep cone in the middle: 3 cells high at its top.
    let hill = Relief::from_fn(level.bounds(), |x, y| {
        let d = ((x * x + y * y) as f64).sqrt();
        (720.0 - 240.0 * d).max(0.0) as i32
    });
    let flat = LevelWay {
        level: &level,
        relief: None,
        terrain: None,
        gait: Gait::UNSKILLED,
    };
    let hilly = LevelWay {
        relief: Some(&hill),
        ..flat
    };
    let (from, to) = (m(-6, 0), m(6, 0));
    let straight = route(&flat, from, to);
    assert_eq!(straight.spots, vec![from, to]);
    let round = route(&hilly, from, to);
    assert!(round.reached);
    assert!(widest(&round) >= 2 * 240, "{:?}", round.spots);
    // Round it is quicker than over it.
    assert!(round.cost < hilly.cost(from, to, false));
}

#[test]
fn good_swimmers_swim_across_a_pool() {
    let mut level = field(6);
    // A pool, wide enough that nobody jumps it.
    for y in -5..=5 {
        for x in -2..=2 {
            level.set(x, y, Cell::Water);
        }
    }
    let way = |gait| LevelWay {
        level: &level,
        relief: None,
        terrain: None,
        gait,
    };
    let (from, to) = (m(-4, 0), m(4, 0));
    let walker = route(&way(swimmer(0)), from, to);
    assert!(walker.reached);
    assert!(
        widest(&walker) >= 6 * 240,
        "round the pool: {:?}",
        walker.spots
    );
    let good = route(&way(swimmer(100)), from, to);
    assert!(good.reached);
    assert_eq!(good.spots, vec![from, to], "straight across");
}
