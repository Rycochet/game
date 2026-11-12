//! Going back into a poi where the player last walked out of it: its first
//! level is set into the wilderness, entered from the world map by a click
//! or Down, and left from there by Up.

use std::collections::{BTreeSet, VecDeque};

use shared::movement::{Step, step_on_level};
use shared::poi::{ChunkPos, Poi};
use shared::protocol::{ClientMessage, ServerMessage};

use super::tests::walk;
use crate::connections::tests::TestClient;
use crate::storage::SiteId;

/// A town near where the player stands, and the player put at it.
fn at_a_town(test: &mut TestClient) -> Poi {
    let (x, y) = test.client.world_position();
    let here = test.world.pois.chunk_at(x, y);
    let poi = (0..20)
        .flat_map(|r| (-r..=r).flat_map(move |dy| (-r..=r).map(move |dx| (dx, dy))))
        .filter_map(|(dx, dy)| {
            let chunk = ChunkPos {
                x: here.x + dx,
                y: here.y + dy,
            };
            test.world.pois.in_chunk(chunk)
        })
        .find(|poi| poi.kind.is_town());
    let poi = poi.expect("a town nearby");
    test.client.character.world_position = (poi.x, poi.y);
    poi
}

fn enter(test: &mut TestClient, poi: &Poi) -> Vec<ServerMessage> {
    test.send(&[ClientMessage::EnterSite { chunk: poi.chunk }])
}

fn position(test: &TestClient) -> (i64, i64) {
    let visit = test.client.visit.as_ref().expect("in the wilderness");
    assert!(visit.is_wilderness());
    visit.position()
}

/// The outside cell of the town, reachable from the player, the most steps
/// away (within 40, to stay in the window).
fn far_in_town(test: &TestClient) -> (i64, i64) {
    let visit = test.client.visit.as_ref().unwrap();
    let level = &visit.level.level;
    let mirror = visit.mirrors.first().expect("the town set in");
    let mut seen = BTreeSet::from([visit.position()]);
    let mut queue = VecDeque::from([(visit.position(), 0)]);
    let mut farthest = None;
    while let Some((at, steps)) = queue.pop_front() {
        if mirror.of(at.0, at.1).is_some() && level.cell(at.0, at.1).is_outside() {
            farthest = Some(at);
        }
        if steps == 40 {
            continue;
        }
        for (dx, dy) in [(1, 0), (-1, 0), (0, 1), (0, -1)] {
            let step = Step::new(dx, dy).unwrap();
            if let Some(next) = step_on_level(level, at, step)
                && seen.insert(next)
            {
                queue.push_back((next, steps + 1));
            }
        }
    }
    farthest.expect("somewhere else in the town")
}

/// Goes in, walks to a far part of the town and leaves by Up: where they
/// first arrived and where they walked out, in the wilderness.
fn walk_out_far(test: &mut TestClient, poi: &Poi) -> ((i64, i64), (i64, i64)) {
    enter(test, poi);
    let arrived = position(test);
    let out = far_in_town(test);
    assert_ne!(out, arrived, "a different way out to test with");
    walk(test, out);
    assert_eq!(position(test), out);
    let answers = test.send(&[ClientMessage::Up]);
    assert!(matches!(
        answers[..],
        [ServerMessage::OnWorldMap { .. }, ..]
    ));
    test.client.character.world_position = (poi.x, poi.y);
    (arrived, out)
}

#[test]
fn going_back_in_arrives_where_they_walked_out() {
    let mut test = TestClient::new();
    let poi = at_a_town(&mut test);
    let (_, out) = walk_out_far(&mut test, &poi);
    let id = SiteId::Poi(poi.chunk);
    let (index, _) = test.client.character.left_at[&id];
    assert_eq!(index, 0, "kept as the town's first level's cell");
    let answers = enter(&mut test, &poi);
    assert_eq!(position(&test), out);
    let arrival = answers.iter().find_map(|message| match message {
        ServerMessage::EnterLevel(info) => Some(info.arrival.cell()),
        _ => None,
    });
    assert_eq!(arrival, Some(out), "the client is told so too");
}

#[test]
fn a_changed_site_starts_at_its_entrance_again() {
    let mut test = TestClient::new();
    let poi = at_a_town(&mut test);
    let (arrived, _) = walk_out_far(&mut test, &poi);
    let id = SiteId::Poi(poi.chunk);
    // As if they knew another version of its map.
    *test.client.character.site_versions.get_mut(&id).unwrap() += 1;
    enter(&mut test, &poi);
    assert_eq!(position(&test), arrived);
}

#[test]
fn a_spot_no_longer_in_it_starts_at_the_entrance() {
    let mut test = TestClient::new();
    let poi = at_a_town(&mut test);
    let (arrived, _) = walk_out_far(&mut test, &poi);
    let id = SiteId::Poi(poi.chunk);
    // Far outside the town.
    test.client
        .character
        .left_at
        .insert(id, (0, (-1000, -1000)));
    enter(&mut test, &poi);
    assert_eq!(position(&test), arrived);
}

#[test]
fn forgetting_the_world_map_forgets_where_they_walked_out() {
    let mut test = TestClient::new();
    let poi = at_a_town(&mut test);
    let (arrived, _) = walk_out_far(&mut test, &poi);
    test.send(&[ClientMessage::DevForgetMap]);
    assert!(test.client.character.left_at.is_empty());
    enter(&mut test, &poi);
    assert_eq!(position(&test), arrived);
}
