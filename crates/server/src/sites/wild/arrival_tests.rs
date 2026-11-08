//! Exploring from a poi arrives outside it, and the client is told which
//! poi's cutout the player stands in as it changes.

use shared::poi::PoiKind;
use shared::protocol::{ClientMessage, ServerMessage};
use shared::site::{Exit, Generator, Site};
use shared::wilderness::{ChunkPlan, Cutout};

use super::arrival::FROM_STAIRS;
use crate::connections::tests::TestClient;

/// The first chunk near the start whose cutout `wanted` picks.
fn cutout_near_start(test: &TestClient, wanted: impl Fn(&Cutout) -> bool) -> Cutout {
    let world = &test.world;
    let (x, y) = test.client.world_position();
    let start = world.pois.chunk_at(x, y);
    (0..16)
        .flat_map(|ring| (-ring..=ring).flat_map(move |dy| (-ring..=ring).map(move |dx| (dx, dy))))
        .map(|(dx, dy)| shared::poi::ChunkPos {
            x: start.x + dx,
            y: start.y + dy,
        })
        .map(|chunk| ChunkPlan::with_new_site(&world.wilderness.wild, &world.pois, chunk))
        .find_map(|plan| plan.cutout().filter(|cutout| wanted(cutout)).cloned())
        .expect("such a poi near the start")
}

/// Explores from the cutout's poi: where the player arrives, and what the
/// client is told of the poi they stand in.
fn explore_from(test: &mut TestClient, cutout: &Cutout) -> ((i64, i64), Option<PoiKind>) {
    test.client.character.world_position = (cutout.poi().x, cutout.poi().y);
    let answers = test.send(&[ClientMessage::Down]);
    let told = answers.iter().find_map(|answer| match answer {
        ServerMessage::InPoi(kind) => Some(*kind),
        _ => None,
    });
    let at = test.client.visit.as_ref().expect("exploring").position();
    (at, told.expect("told which poi"))
}

fn is_cave(cutout: &Cutout) -> bool {
    matches!(Generator::for_poi(cutout.poi().kind), Generator::Cave(_))
}

#[test]
fn exploring_from_a_building_arrives_at_its_entrance() {
    let mut test = TestClient::new();
    let cutout = cutout_near_start(&test, |cutout| {
        cutout.offset().is_some()
            && Site::new(cutout.poi().kind, cutout.poi().seed).is_building()
            && Generator::for_site(cutout.poi().kind, cutout.poi().seed, cutout.poi().desert).ruin()
                == 0
    });
    let (ox, oy) = cutout.offset().unwrap();
    let site = Site::new(cutout.poi().kind, cutout.poi().seed);
    let (ex, ey) = site.level(0).arrival(Exit::WorldMap).unwrap();
    let (at, told) = explore_from(&mut test, &cutout);
    assert_eq!(at, (ex + ox, ey + oy));
    assert_eq!(told, Some(cutout.poi().kind));
}

#[test]
fn exploring_from_a_way_down_arrives_beside_it_and_walking_off_is_told() {
    let mut test = TestClient::new();
    let cutout = cutout_near_start(&test, |cutout| cutout.offset().is_none());
    let stairs = cutout.stairs()[0];
    let (at, told) = explore_from(&mut test, &cutout);
    assert_eq!(at, (stairs.x, stairs.y + FROM_STAIRS));
    assert_eq!(told, Some(cutout.poi().kind));
    // South, out of the clearing: told once, on the step out.
    let answers = test.steps(&[(0, 1), (0, 1), (0, 1)]);
    let told: Vec<_> = answers
        .iter()
        .filter(|answer| matches!(answer, ServerMessage::InPoi(_)))
        .collect();
    assert_eq!(told, [&ServerMessage::InPoi(None)]);
}

#[test]
fn exploring_from_a_cave_mouth_arrives_outside_its_hill() {
    let mut test = TestClient::new();
    let cutout = cutout_near_start(&test, |cutout| cutout.offset().is_some() && is_cave(cutout));
    let (at, told) = explore_from(&mut test, &cutout);
    assert!(!cutout.contains(at.0, at.1), "{at:?} is in the hill");
    assert_eq!(told, None);
    let (min_x, min_y, width, height) = cutout.level().bounds();
    assert!((min_x..min_x + width).contains(&at.0) && (min_y..min_y + height).contains(&at.1));
}
