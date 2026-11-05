//! Going back into a site where the player last walked out of it.

use std::collections::{BTreeSet, VecDeque};

use shared::movement::{Step, step_on_level};
use shared::poi::{ChunkPos, Poi};
use shared::protocol::{ClientMessage, ServerMessage};
use shared::site::leads_out;

use super::tests::walk;
use crate::connections::tests::TestClient;
use crate::storage::SiteId;

/// A poi near where the player stands, and the player put at it.
fn at_a_poi(test: &mut TestClient) -> Poi {
    let (x, y) = test.client.world_position();
    let here = test.world.pois.chunk_at(x, y);
    let poi = (0..20)
        .flat_map(|r| (-r..=r).flat_map(move |dy| (-r..=r).map(move |dx| (dx, dy))))
        .find_map(|(dx, dy)| {
            let chunk = ChunkPos {
                x: here.x + dx,
                y: here.y + dy,
            };
            test.world.pois.in_chunk(chunk)
        })
        .expect("a poi nearby");
    test.client.character.world_position = (poi.x, poi.y);
    poi
}

fn enter(test: &mut TestClient, poi: &Poi) -> Vec<ServerMessage> {
    test.send(&[ClientMessage::EnterSite { chunk: poi.chunk }])
}

fn position(test: &TestClient) -> (u32, (i64, i64)) {
    let visit = test.client.visit.as_ref().expect("in the site");
    (visit.index, visit.position)
}

/// The edge cell reachable from the player that is the most steps away.
fn far_way_out(test: &TestClient) -> (i64, i64) {
    let visit = test.client.visit.as_ref().unwrap();
    let level = &visit.level.level;
    let mut seen = BTreeSet::from([visit.position]);
    let mut queue = VecDeque::from([visit.position]);
    let mut farthest = None;
    while let Some(at) = queue.pop_front() {
        if leads_out(level, at.0, at.1) {
            farthest = Some(at);
        }
        for (dx, dy) in [(1, 0), (-1, 0), (0, 1), (0, -1)] {
            let step = Step::new(dx, dy).unwrap();
            if let Some(next) = step_on_level(level, at, step)
                && seen.insert(next)
            {
                queue.push_back(next);
            }
        }
    }
    farthest.expect("a way out")
}

/// Enters, walks out at a far edge, and returns where they first arrived
/// and where they walked out.
fn walk_out_far(test: &mut TestClient, poi: &Poi) -> ((i64, i64), (i64, i64)) {
    enter(test, poi);
    let (_, arrived) = position(test);
    let out = far_way_out(test);
    assert_ne!(out, arrived, "a different way out to test with");
    walk(test, out);
    assert_eq!(position(test).1, out);
    let answers = test.send(&[ClientMessage::UseExit]);
    assert!(matches!(
        answers[..],
        [ServerMessage::OnWorldMap { .. }, ..]
    ));
    (arrived, out)
}

#[test]
fn going_back_in_arrives_where_they_walked_out() {
    let mut test = TestClient::new();
    let poi = at_a_poi(&mut test);
    let (_, out) = walk_out_far(&mut test, &poi);
    let id = SiteId::Poi(poi.chunk);
    assert_eq!(test.client.character.left_at.get(&id), Some(&(0, out)));
    let answers = enter(&mut test, &poi);
    assert_eq!(position(&test), (0, out));
    let arrival = answers.iter().find_map(|message| match message {
        ServerMessage::EnterLevel(info) => Some(info.arrival),
        _ => None,
    });
    assert_eq!(arrival, Some(out), "the client is told so too");
}

#[test]
fn a_changed_site_starts_at_its_entrance_again() {
    let mut test = TestClient::new();
    let poi = at_a_poi(&mut test);
    let (arrived, _) = walk_out_far(&mut test, &poi);
    let id = SiteId::Poi(poi.chunk);
    // As if they knew another version of its map.
    *test.client.character.site_versions.get_mut(&id).unwrap() += 1;
    enter(&mut test, &poi);
    assert_eq!(position(&test), (0, arrived));
    assert!(!test.client.character.left_at.contains_key(&id));
}

#[test]
fn a_spot_no_longer_walkable_starts_at_the_entrance() {
    let mut test = TestClient::new();
    let poi = at_a_poi(&mut test);
    let (arrived, _) = walk_out_far(&mut test, &poi);
    let id = SiteId::Poi(poi.chunk);
    // Far outside the level: never walkable.
    test.client
        .character
        .left_at
        .insert(id, (0, (-1000, -1000)));
    enter(&mut test, &poi);
    assert_eq!(position(&test), (0, arrived));
}

#[test]
fn forgetting_the_world_map_forgets_where_they_walked_out() {
    let mut test = TestClient::new();
    let poi = at_a_poi(&mut test);
    let (arrived, _) = walk_out_far(&mut test, &poi);
    test.send(&[ClientMessage::DevForgetMap]);
    assert!(test.client.character.left_at.is_empty());
    enter(&mut test, &poi);
    assert_eq!(position(&test), (0, arrived));
}
