use std::collections::BTreeSet;

use shared::poi::ChunkPos;
use shared::protocol::ClientMessage;

use super::*;
use crate::connections::tests::TestClient;

fn chunks(answers: &[ServerMessage]) -> Vec<ChunkPos> {
    answers
        .iter()
        .filter_map(|answer| match answer {
            ServerMessage::WorldChunk(chunk) => Some(chunk.chunk),
            _ => None,
        })
        .collect()
}

#[test]
fn chunks_in_sight_are_sent_on_joining_and_once() {
    let mut test = TestClient::new();
    let arrived = test.client.world_position();
    let first = chunks(&test.welcome);
    // Sight plus preload either way, 64 points per chunk: 11-12 chunks.
    let row = arrived.1.div_euclid(64);
    let side = first.iter().filter(|c| c.y == row).count();
    assert!((11..=13).contains(&side), "{side}");
    assert_eq!(first.len(), side * side);
    // Stepping there and back sends nothing new.
    assert!(chunks(&test.steps(&[(1, 0), (-1, 0)])).is_empty());
    assert_eq!(test.client.world_position(), arrived);
}

#[test]
fn forgetting_keeps_only_what_is_in_sight() {
    let mut test = TestClient::new();
    // Far off, then back: the first place is out of sight.
    test.send(&[ClientMessage::DevTeleport { x: 6400, y: 0 }]);
    let far = ChunkPos { x: 100, y: 0 };
    test.send(&[ClientMessage::DevTeleport { x: 0, y: 0 }]);
    assert!(test.client.character.pois.contains_key(&far));

    let answers = test.send(&[ClientMessage::DevForgetMap]);
    assert_eq!(answers[0], ServerMessage::ForgetWorldMap);
    assert!(chunks(&answers).contains(&ChunkPos { x: 0, y: 0 }));
    assert!(!chunks(&answers).contains(&far));
    assert!(!test.client.character.pois.contains_key(&far));
}

#[test]
fn steps_into_water_are_refused() {
    let mut test = TestClient::new();
    let ground = &test.world.ground;
    let conditions = test.world.conditions();
    let (x, y) = (0..4000)
        .flat_map(|y| (0..4000).map(move |x| (x, y)))
        .find(|&(x, y)| {
            ground.elevation(x, y, conditions).is_some()
                && ground.elevation(x + 1, y, conditions).is_none()
        })
        .expect("a shore somewhere");
    test.send(&[ClientMessage::DevTeleport { x, y }]);
    test.steps(&[(1, 0)]);
    assert_eq!(test.client.world_position(), (x, y));
}

#[test]
fn moving_sends_only_the_new_chunks() {
    let mut test = TestClient::new();
    let first = chunks(&test.welcome);
    let more = chunks(&test.walk_to((0, 64)));
    assert!(!more.is_empty());
    let all: BTreeSet<_> = first.iter().chain(&more).collect();
    assert_eq!(all.len(), first.len() + more.len());
}

#[test]
fn dev_teleports_say_where_and_send_what_is_there() {
    let mut test = TestClient::new();
    let answers = test.send(&[ClientMessage::DevTeleport { x: 5000, y: 20 }]);
    assert_eq!(answers[0], ServerMessage::Teleported { x: 5000, y: 20 });
    assert!(!chunks(&answers).is_empty());
    // And back to the start: the World Gate nearest.
    let gate = nearest_gate(&test.world, (5000, 20));
    let answers = test.send(&[ClientMessage::DevBackToStart]);
    let (x, y) = gate;
    assert_eq!(answers[0], ServerMessage::OnWorldMap { x, y });
    assert_eq!(test.client.world_position(), gate);
}

#[test]
fn new_characters_arrive_through_a_world_gate() {
    let test = TestClient::new();
    let at = test.client.world_position();
    let gates = &test.world.pois.world_gates().gates;
    assert!(gates.iter().any(|gate| (gate.x, gate.y) == at), "{at:?}");
}

#[test]
fn dev_tools_are_ignored_where_not_allowed() {
    let mut test = TestClient::new();
    test.dev_tools = false;
    // Not the host, who is the Owner and may use them anyway.
    test.client.host = false;
    let arrived = test.client.world_position();
    assert!(
        test.send(&[ClientMessage::DevTeleport { x: 50, y: 0 }])
            .is_empty()
    );
    assert_eq!(test.client.world_position(), arrived);
}

#[test]
fn a_new_connection_is_sent_all_the_character_knows() {
    let mut test = TestClient::new();
    let at_arrival: BTreeSet<_> = test.client.character.pois.keys().copied().collect();
    test.send(&[ClientMessage::DevTeleport { x: 5000, y: 20 }]);
    let known: BTreeSet<_> = test.client.character.pois.keys().copied().collect();
    assert!(!at_arrival.is_empty() && at_arrival.is_subset(&known));
    // As it is when a character joins again.
    test.client.world_chunks.clear();
    let TestClient { client, world, .. } = &mut test;
    assert!(look_around(client, world));
    let sent: BTreeSet<_> = chunks(&test.send(&[])).into_iter().collect();
    assert!(known.is_subset(&sent));
}

/// The pois a client was told of in these answers.
fn pois(answers: &[ServerMessage]) -> Vec<shared::protocol::SeenPoi> {
    answers
        .iter()
        .filter_map(|answer| match answer {
            ServerMessage::WorldChunk(chunk) => chunk.poi,
            _ => None,
        })
        .collect()
}

/// A chunk in reach whose poi isn't a town.
fn hidden_chunk(test: &TestClient) -> ChunkPos {
    let hidden = |chunk: &&ChunkPos| {
        let poi = test.world.pois.in_chunk(**chunk);
        poi.is_some_and(|poi| !poi.kind.is_town())
    };
    *test.client.world_chunks.iter().find(hidden).unwrap()
}

#[test]
fn only_towns_are_seen_until_a_poi_is_known() {
    let mut test = TestClient::new();
    test.client.reveal_pois = false;
    let chunk = hidden_chunk(&test);
    test.client.world_chunks.clear();
    let TestClient { client, world, .. } = &mut test;
    assert!(look_around(client, world));
    let seen = pois(&test.send(&[]));
    assert!(seen.iter().all(|poi| poi.kind.is_town()));

    // Once learned of, it is sent.
    test.client.character.pois.entry(chunk).or_default().known = true;
    test.client.world_chunks.clear();
    let TestClient { client, world, .. } = &mut test;
    assert!(look_around(client, world));
    assert!(pois(&test.send(&[])).iter().any(|poi| poi.chunk == chunk));
}

#[test]
fn the_key_goes_to_the_showcase_and_esc_comes_back() {
    let mut test = TestClient::new();
    let answers = test.send(&[ClientMessage::DevToShowcase]);
    let (x, y) = showcase::ARRIVAL;
    assert_eq!(answers[0], ServerMessage::OnWorldMap { x, y });
    assert!(
        pois(&answers)
            .iter()
            .any(|poi| poi.kind == shared::poi::PoiKind::ALL[0])
    );
    let answers = test.send(&[ClientMessage::DevBackToStart]);
    let (x, y) = nearest_gate(&test.world, showcase::ARRIVAL);
    assert_eq!(answers[0], ServerMessage::OnWorldMap { x, y });
}

/// What ctrl+click on the full map sends: from anywhere on the world map,
/// into the poi.
#[test]
fn teleporting_to_a_poi_then_entering_goes_in() {
    let mut test = TestClient::new();
    let chunk = ChunkPos { x: 3, y: 1000 };
    let poi = test.world.pois.in_chunk(chunk).unwrap();
    test.send(&[
        ClientMessage::DevTeleport { x: poi.x, y: poi.y },
        ClientMessage::EnterSite { chunk },
    ]);
    assert!(test.client.visit.is_some());
}

#[test]
fn dev_tools_reveal_every_poi() {
    let mut test = TestClient::new();
    test.client.reveal_pois = false;
    let answers = test.send(&[ClientMessage::DevRevealPois]);
    assert!(pois(&answers).iter().any(|poi| !poi.kind.is_town()));
}

#[test]
fn a_poi_not_known_of_cannot_be_entered() {
    let mut test = TestClient::new();
    test.client.reveal_pois = false;
    let chunk = hidden_chunk(&test);
    let poi = test.world.pois.in_chunk(chunk).unwrap();
    test.send(&[ClientMessage::DevTeleport { x: poi.x, y: poi.y }]);
    test.send(&[ClientMessage::EnterSite { chunk }]);
    assert!(test.client.visit.is_none());
}

#[test]
fn pois_in_sight_are_known() {
    let test = TestClient::new();
    let (_, arrived_y) = test.client.world_position();
    let known = &test.client.character.pois;
    assert!(!known.is_empty());
    // Only those in sight, not the preload margin beyond it.
    for chunk in known.keys() {
        // (East-west, chunk numbers wrap round the planet.)
        assert!((chunk.y * 64 - arrived_y).abs() <= WORLD_MAP_SIGHT + 64);
    }
    assert!(known.len() < test.client.world_chunks.len());
}
