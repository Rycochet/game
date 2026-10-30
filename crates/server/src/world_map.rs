//! Sending each player the world map chunks coming into their sight, and
//! remembering the features they have seen.

use shared::movement::{Step, step_on_world_map};
use shared::protocol::{ServerMessage, WORLD_MAP_PRELOAD, WORLD_MAP_SIGHT, WorldChunk};

use crate::connections::Client;
use crate::storage::KnownFeature;
use crate::{ServerWorld, sites};

/// The player takes a step on the world map (anywhere goes, so far). `false`
/// if the client has gone.
pub(crate) fn stepped(client: &mut Client, world: &ServerWorld, step: Step) -> bool {
    client.character.world_position = step_on_world_map(client.world_position(), step);
    client.unsaved = true;
    look_around(client, world)
}

/// Where new characters start on the world map (`CharacterSave`'s default).
pub(crate) const START: (i64, i64) = (0, 0);

/// Developer tools: puts the player at `(x, y)` on the world map.
pub(crate) fn teleport(client: &mut Client, world: &ServerWorld, x: i64, y: i64) -> bool {
    client.character.world_position = (x, y);
    client.unsaved = true;
    client.changed_map();
    client.link.send(&ServerMessage::Teleported { x, y }) && look_around(client, world)
}

/// Developer tools: puts the player back where new characters start, out of
/// any site.
pub(crate) fn back_to_start(client: &mut Client, world: &mut ServerWorld) -> bool {
    sites::leave(client, world);
    client.visit = None;
    let (x, y) = START;
    client.character.world_position = START;
    client.unsaved = true;
    client.changed_map();
    client.link.send(&ServerMessage::OnWorldMap { x, y }) && look_around(client, world)
}

/// The player is somewhere new on the world map: the features now in sight
/// are known from now on, and the chunks within sight, plus a margin, that
/// the client hasn't been sent yet are sent. `false` if the client has
/// gone.
pub(crate) fn look_around(client: &mut Client, world: &ServerWorld) -> bool {
    let (x, y) = client.world_position();
    let sight = WORLD_MAP_SIGHT;
    for chunk in world
        .features
        .chunks_in_area((x - sight, y - sight), (x + sight, y + sight))
    {
        let known = &mut client.character.features;
        if world.features.in_chunk(chunk).is_some() && !known.contains_key(&chunk) {
            known.insert(chunk, KnownFeature::default());
            client.unsaved = true;
        }
    }
    let reach = WORLD_MAP_SIGHT + WORLD_MAP_PRELOAD;
    let chunks = world
        .features
        .chunks_in_area((x - reach, y - reach), (x + reach, y + reach));
    for chunk in chunks {
        if client.world_chunks.insert(chunk) {
            let feature = world.features.in_chunk(chunk).map(Into::into);
            if !client
                .link
                .send(&ServerMessage::WorldChunk(WorldChunk { chunk, feature }))
            {
                return false;
            }
        }
    }
    true
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeSet;

    use shared::feature::ChunkPos;
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
        let first = chunks(&test.welcome);
        // Sight plus preload either way, 64 points per chunk: 11-12 chunks.
        let side = first.iter().filter(|c| c.y == 0).count();
        assert!((11..=13).contains(&side), "{side}");
        assert_eq!(first.len(), side * side);
        // Stepping there and back sends nothing new.
        assert!(chunks(&test.steps(&[(1, 0), (-1, 0)])).is_empty());
        assert_eq!(test.client.world_position(), (0, 0));
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
        // And back to the start.
        let answers = test.send(&[ClientMessage::DevBackToStart]);
        let (x, y) = START;
        assert_eq!(answers[0], ServerMessage::OnWorldMap { x, y });
        assert_eq!(test.client.world_position(), START);
    }

    #[test]
    fn dev_tools_are_ignored_where_not_allowed() {
        let mut test = TestClient::new();
        test.dev_tools = false;
        assert!(
            test.send(&[ClientMessage::DevTeleport { x: 50, y: 0 }])
                .is_empty()
        );
        assert_eq!(test.client.world_position(), START);
    }

    #[test]
    fn features_in_sight_are_known() {
        let test = TestClient::new();
        let known = &test.client.character.features;
        assert!(!known.is_empty());
        // Only chunks with a feature, and only those in sight, not the
        // preload margin beyond it.
        for chunk in known.keys() {
            assert!(test.world.features.in_chunk(*chunk).is_some());
            // (East-west, chunk numbers wrap round the planet.)
            assert!(chunk.y.abs() * 64 <= WORLD_MAP_SIGHT + 64);
        }
        assert!(known.len() < test.client.world_chunks.len());
    }
}
