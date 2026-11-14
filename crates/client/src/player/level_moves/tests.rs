use bevy::ecs::system::RunSystemOnce;
use shared::creature::Size;
use shared::dungeon::Cell;

use super::*;

/// A floor room from `(-r, -r)` to `(r, r)`, walled round.
fn room(r: i64) -> Level {
    let mut level = Level::new(-r - 1, -r - 1, 2 * r + 3, 2 * r + 3);
    for y in -r..=r {
        for x in -r..=r {
            level.set(x, y, Cell::Floor);
        }
    }
    level
}

/// Follows a route two cells east with `distance` to go on ground
/// `slowness` times as slow as flat, returning whether it arrived and where
/// the player is.
fn follow(distance: f64, slowness: f64) -> (bool, Spot) {
    let mut world = World::new();
    world.init_resource::<Messages<AskDoor>>();
    let system = move |mut asks: MessageWriter<AskDoor>| {
        let level = room(4);
        let walker = Walker {
            level: &level,
            size: Size::Medium,
            swims: false,
            others: &[],
            squeezes: true,
        };
        let start = Spot::middle_of((-1, 0));
        let mut travel = Travel {
            stride: Stride::at(start),
            route_arrives: true,
            ..default()
        };
        travel.spots.push_back(Spot::middle_of((1, 0)));
        let mut moves = Vec::new();
        let slow = |_, _| slowness;
        let arrived = follow_route(&mut travel, &walker, distance, slow, &mut moves, &mut asks);
        (arrived, travel.stride.at)
    };
    world.run_system_once(system).unwrap()
}

#[test]
fn a_long_enough_walk_arrives() {
    let (arrived, at) = follow(2000.0, 1.0);
    assert!(arrived);
    assert_eq!(at, Spot::middle_of((1, 0)));
}

/// A leftover too small to walk once divided by the ground's slowness used to
/// loop for ever (the client froze).
#[test]
fn a_leftover_too_small_to_walk_stops() {
    let smallest = f64::from_bits(1);
    let (arrived, at) = follow(smallest, 3.0);
    assert!(!arrived);
    assert_eq!(at, Spot::middle_of((-1, 0)));
}

#[test]
fn a_distance_that_is_not_a_number_stops() {
    let (arrived, _) = follow(f64::NAN, 1.0);
    assert!(!arrived);
}

#[test]
fn an_endless_distance_stops() {
    let (arrived, _) = follow(f64::INFINITY, 1.0);
    assert!(!arrived);
}
