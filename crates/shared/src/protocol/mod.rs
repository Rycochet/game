//! Messages between the client and the server, and the `Link` that carries
//! them. See `docs/context/shared/protocol.md`.
//!
//! Clients send intents (`ClientMessage`); the server answers with what the
//! player may know (`ServerMessage`). Every message is encoded with postcard,
//! also in single player, where the link is an in-process channel.

mod link;

pub use link::{ClientLink, Link, LinkError, ServerLink, local_pair};

use serde::{Deserialize, Serialize};

use crate::feature::{CHUNK_SIZE, ChunkPos, Feature};
use crate::rng::Seed;
use crate::world::WorldId;

/// Bump whenever a message changes, so mismatched clients and servers refuse
/// each other instead of misreading messages.
pub const PROTOCOL_VERSION: u32 = 2;

/// How far from the player (east–west and north–south, in grid points) the
/// client may show anything on the world map: the corners of its window
/// turned 45°, plus a token.
pub const WORLD_MAP_SIGHT: i64 = 300;

/// How much further than `WORLD_MAP_SIGHT` the server sends world map
/// chunks, so the client already has them when they come into sight.
pub const WORLD_MAP_PRELOAD: i64 = CHUNK_SIZE;

/// The client reports its world map position again once it is this far
/// (either way) from the last one it reported. Must stay below
/// `WORLD_MAP_PRELOAD`, so nothing comes into sight before it is sent.
pub const WORLD_MAP_REPORT_DISTANCE: i64 = 16;

/// From a client to the server.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum ClientMessage {
    /// The first message on a new connection.
    Hello { protocol: u32 },
    /// TEMPORARY, until movement goes through the server: where the player
    /// is on the world map, so the server knows which chunks to send.
    WorldMapPosition { x: i64, y: i64 },
}

/// From the server to a client.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum ServerMessage {
    /// The answer to `Hello`: the world the player is in.
    Welcome(WorldInfo),
    /// The server won't talk to this client, and why.
    Refused(String),
    /// A world map chunk, as this player knows it.
    WorldChunk(WorldChunk),
}

/// What a player knows of one world map chunk. The terrain itself isn't
/// sent: the client generates it from the terrain seed. Later this also
/// carries the chunk's changes and per-player state (visited, attitude).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct WorldChunk {
    pub chunk: ChunkPos,
    /// Its feature, if it has one. TEMPORARY: until the server sends sites,
    /// this includes the feature's secret map seed, so the client can
    /// generate the site itself.
    pub feature: Option<Feature>,
}

/// What a client is told about a world when it joins.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct WorldInfo {
    pub id: WorldId,
    /// Public: the client generates the world map terrain from it.
    pub terrain_seed: Seed,
    /// Which world map generator the world was made with (`WORLD_MAP_VERSION`).
    pub world_map_version: u32,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::feature::FeatureKind;

    #[test]
    fn positions_are_reported_before_chunks_come_into_sight() {
        const { assert!(WORLD_MAP_REPORT_DISTANCE < WORLD_MAP_PRELOAD) };
    }

    /// One of every message, so a message that can't be sent fails here.
    fn every_message() -> (Vec<ClientMessage>, Vec<ServerMessage>) {
        let client = vec![
            ClientMessage::Hello {
                protocol: PROTOCOL_VERSION,
            },
            ClientMessage::WorldMapPosition { x: -3, y: 4 },
        ];
        let server = vec![
            ServerMessage::Welcome(WorldInfo {
                id: WorldId(7),
                terrain_seed: Seed::new(8),
                world_map_version: 1,
            }),
            ServerMessage::Refused("no".into()),
            ServerMessage::WorldChunk(WorldChunk {
                chunk: ChunkPos { x: 1, y: -2 },
                feature: Some(Feature {
                    kind: FeatureKind::Castle,
                    chunk: ChunkPos { x: 1, y: -2 },
                    x: 70,
                    y: -100,
                    seed: Seed::new(9),
                }),
            }),
            ServerMessage::WorldChunk(WorldChunk {
                chunk: ChunkPos { x: 0, y: 0 },
                feature: None,
            }),
        ];
        (client, server)
    }

    #[test]
    fn every_message_survives_the_link() {
        let (client, server) = local_pair();
        let (to_server, to_client) = every_message();
        for message in &to_server {
            assert!(client.send(message));
            assert_eq!(server.receive(), Ok(Some(message.clone())));
        }
        for message in &to_client {
            assert!(server.send(message));
            assert_eq!(client.receive(), Ok(Some(message.clone())));
        }
        assert_eq!(client.receive(), Ok(None));
    }

    #[test]
    fn a_link_knows_when_the_other_end_has_gone() {
        let (client, server) = local_pair();
        drop(server);
        assert_eq!(client.receive(), Err(LinkError::Closed));
        assert!(!client.send(&ClientMessage::Hello { protocol: 1 }));
    }
}
