//! Pois stop being secret in the wilderness on their way in being seen, or
//! the player stepping inside them.

use shared::dungeon::line_is_clear;
use shared::poi::Poi;
use shared::protocol::{ClientMessage, ServerMessage};
use shared::wilderness::Cutout;

use super::tests::{chunk_near_start, position};
use crate::connections::tests::TestClient;

/// A client who knows of no poi but towns, and a poi set in near the start
/// that `wanted` picks.
fn secret(wanted: impl Fn(&Cutout) -> bool) -> (TestClient, Poi, Cutout) {
    let mut test = TestClient::new();
    test.client.reveal_pois = false;
    let plan = chunk_near_start(&test, wanted);
    let cutout = plan.cutout().unwrap().clone();
    (test, *cutout.poi(), cutout)
}

fn found_in(answers: &[ServerMessage]) -> Option<shared::poi::PoiKind> {
    answers.iter().find_map(|answer| match answer {
        &ServerMessage::PoiFound { kind } => Some(kind),
        _ => None,
    })
}

fn known(test: &TestClient, poi: &Poi) -> bool {
    let seen = test.client.character.pois.get(&poi.chunk);
    seen.is_some_and(|seen| seen.known)
}

#[test]
fn a_secret_poi_stood_in_is_found_and_its_border_shown() {
    let (mut test, poi, cutout) = secret(|c| !c.poi().kind.is_town() && c.offset().is_some());
    test.client.character.world_position = (poi.x, poi.y);
    let answers = test.send(&[ClientMessage::Down]);
    assert!(known(&test, &poi));
    assert_eq!(found_in(&answers), Some(poi.kind));
    let border = answers.iter().rev().find_map(|answer| match answer {
        ServerMessage::Borders(cells) => Some(cells.clone()),
        _ => None,
    });
    let border = border.expect("sent again once found");
    assert!(cutout.border().iter().all(|cell| border.contains(cell)));
    // Found once only.
    let (x, y) = position(&test);
    let answers = test.send(&[ClientMessage::DevTeleport { x, y: y + 1 }]);
    assert_eq!(found_in(&answers), None);
}

#[test]
fn seeing_its_way_in_from_outside_finds_a_poi() {
    let (mut test, poi, cutout) = secret(|c| !c.poi().kind.is_town() && c.offset().is_some());
    // Four world map points south: well out of sight of it.
    test.client.character.world_position = (poi.x, poi.y + 4);
    test.send(&[ClientMessage::Down]);
    assert!(!known(&test, &poi), "out of sight");
    let entrance = test
        .world
        .wilderness
        .chunk(poi.chunk)
        .unwrap()
        .entrance
        .unwrap();
    let level = &test.client.visit.as_ref().unwrap().level.level;
    let (min_x, min_y, width, height) = cutout.level().bounds();
    let outside = |&(x, y): &(i64, i64)| {
        !((min_x..min_x + width).contains(&x) && (min_y..min_y + height).contains(&y))
    };
    let spot = (-12..=12)
        .flat_map(|dy| (-12..=12).map(move |dx| (entrance.0 + dx, entrance.1 + dy)))
        .filter(outside)
        .find(|&(x, y)| level.is_walkable(x, y) && line_is_clear(level, (x, y), entrance))
        .expect("open ground in sight of the way in");
    let answers = test.send(&[ClientMessage::DevTeleport {
        x: spot.0,
        y: spot.1,
    }]);
    assert!(known(&test, &poi));
    assert_eq!(found_in(&answers), Some(poi.kind));
}

#[test]
fn finding_a_town_tells_nothing() {
    let (mut test, poi, _) = secret(|c| c.poi().kind.is_town());
    test.client.character.world_position = (poi.x, poi.y);
    let answers = test.send(&[ClientMessage::Down]);
    assert!(known(&test, &poi));
    assert_eq!(found_in(&answers), None, "everyone knows of towns");
}
