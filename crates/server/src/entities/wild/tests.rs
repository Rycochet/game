use std::collections::BTreeSet;
use std::time::{Duration, Instant};

use shared::protocol::ClientMessage;

use super::*;
use crate::connections::tests::TestClient;

/// A client in the wilderness at the start (in woods, where wolves live), with
/// the areas worked out.
fn exploring() -> TestClient {
    let world = crate::ServerWorld::new(shared::world::WorldSeed::new(12345));
    let mut test = TestClient::of(world);
    test.send(&[ClientMessage::Explore]);
    areas_tick(&mut test);
    test
}

fn areas_tick(test: &mut TestClient) {
    keep_wild_areas(std::slice::from_mut(&mut test.client), &mut test.world);
}

/// The ids of the mobs in the player's area.
fn mobs_about(test: &TestClient) -> BTreeSet<shared::id::Id> {
    let visit = test.client.visit.as_ref().expect("in the wilderness");
    let population = &test.world.populations.0[&visit.population()];
    population.mobs.iter().map(|mob| mob.id).collect()
}

#[test]
fn a_player_in_the_wilderness_has_an_area_with_its_mobs() {
    let test = exploring();
    let visit = test.client.visit.as_ref().unwrap();
    assert_ne!(visit.wild_area(), 0);
    let areas = &test.world.wilderness.areas;
    let held = areas
        .owner
        .values()
        .filter(|&&a| a == visit.wild_area())
        .count();
    assert_eq!(held, 25, "5 × 5 blocks");
    assert!(
        !mobs_about(&test).is_empty(),
        "something lives round the start"
    );
}

#[test]
fn mobs_kept_while_away_come_back_as_they_were_and_go_after_a_while() {
    let mut test = exploring();
    let before = mobs_about(&test);
    test.send(&[ClientMessage::LeaveWilderness]);
    areas_tick(&mut test);
    assert!(test.world.wilderness.areas.areas.is_empty());
    let kept: usize = test
        .world
        .wilderness
        .areas
        .kept
        .values()
        .map(|(m, _)| m.len())
        .sum();
    assert_eq!(kept, before.len());
    // Back within the time: the same mobs, not new ones.
    test.send(&[ClientMessage::Explore]);
    areas_tick(&mut test);
    assert_eq!(mobs_about(&test), before);
    // Away for longer: let go, and made again at home (the same ones, as
    // they are worked out from the seed).
    test.send(&[ClientMessage::LeaveWilderness]);
    areas_tick(&mut test);
    let later = Instant::now() + WILD_BLOCK_KEPT + Duration::from_secs(1);
    blocks::forget_old(&mut test.world, later);
    assert!(test.world.wilderness.areas.kept.is_empty());
    assert!(test.world.wilderness.areas.living.is_empty());
    test.send(&[ClientMessage::Explore]);
    areas_tick(&mut test);
    assert_eq!(mobs_about(&test), before);
}

#[test]
fn walking_away_moves_the_area_with_the_player() {
    let mut test = exploring();
    let area = test.client.visit.as_ref().unwrap().wild_area();
    let first = test.world.wilderness.areas.areas[&area];
    // Two blocks east, out of the middle 3 × 3.
    let (x, y) = test.client.visit.as_ref().unwrap().position();
    let visit = test.client.visit.as_mut().unwrap();
    visit.move_for_test((x + 2 * shared::wilderness::BLOCK_SIZE, y));
    areas_tick(&mut test);
    let visit = test.client.visit.as_ref().unwrap();
    assert_eq!(visit.wild_area(), area, "a lone player's area follows them");
    let now = test.world.wilderness.areas.areas[&area];
    assert_ne!(now, first);
    // The blocks left behind are kept, not held.
    let held: Vec<_> = test.world.wilderness.areas.owner.keys().copied().collect();
    assert!(!held.contains(&BlockPos {
        x: first.x - 2,
        y: first.y
    }));
}
