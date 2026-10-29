//! Sending each player the world map chunks coming into their sight.

use shared::protocol::{ServerMessage, WORLD_MAP_PRELOAD, WORLD_MAP_SIGHT, WorldChunk};

use crate::ServerWorld;
use crate::connections::Client;

/// The player says they are at `(x, y)` on the world map (ignored inside a
/// site): sends the chunks within sight, plus a margin, that the client
/// hasn't been sent yet. `false` if the client has gone.
pub(crate) fn moved_to(client: &mut Client, world: &ServerWorld, x: i64, y: i64) -> bool {
    if client.visit.is_some() {
        return true;
    }
    client.world_position = Some((x, y));
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
    fn chunks_in_sight_are_sent_once() {
        let at = ClientMessage::WorldMapPosition { x: 0, y: 0 };
        let mut test = TestClient::new();
        let first = chunks(&test.send(std::slice::from_ref(&at)));
        // Sight plus preload either way, 64 points per chunk: 11-12 chunks.
        let side = first.iter().filter(|c| c.y == 0).count();
        assert!((11..=13).contains(&side), "{side}");
        assert_eq!(first.len(), side * side);
        // Reporting the same place again sends nothing new.
        assert!(chunks(&test.send(&[at])).is_empty());
    }

    #[test]
    fn moving_sends_only_the_new_chunks() {
        let mut test = TestClient::new();
        let first = chunks(&test.send(&[ClientMessage::WorldMapPosition { x: 0, y: 0 }]));
        let more = chunks(&test.send(&[ClientMessage::WorldMapPosition { x: 0, y: 64 }]));
        assert!(!more.is_empty());
        let all: BTreeSet<_> = first.iter().chain(&more).collect();
        assert_eq!(all.len(), first.len() + more.len());
    }
}
