//! Messages between the client and the server, and the `Link` that carries
//! them. See `docs/context/shared/protocol.md`.
//!
//! Clients send intents (`ClientMessage`); the server answers with what the
//! player may know (`ServerMessage`). Every message is encoded with postcard,
//! also in single player, where the link is an in-process channel.

mod level;
mod link;

pub use level::{
    ENTER_REACH, LEVEL_CHUNK_SIZE, LEVEL_PRELOAD_STEPS, LevelChunk, LevelInfo, SeenStairs,
    level_chunk,
};
pub use link::{ClientLink, Link, LinkError, ServerLink, local_pair};

use serde::{Deserialize, Serialize};

use crate::feature::{CHUNK_SIZE, ChunkPos, Feature, FeatureKind};
use crate::movement::Step;
use crate::rng::Seed;
use crate::world::WorldId;

/// Bump whenever a message changes, so mismatched clients and servers refuse
/// each other instead of misreading messages.
pub const PROTOCOL_VERSION: u32 = 5;

/// How far from the player (east–west and north–south, in grid points) the
/// client may show anything on the world map: the corners of its window
/// turned 45°, plus a token.
pub const WORLD_MAP_SIGHT: i64 = 300;

/// How much further than `WORLD_MAP_SIGHT` the server sends world map
/// chunks, so the client already has them when they come into sight.
pub const WORLD_MAP_PRELOAD: i64 = CHUNK_SIZE;

/// From a client to the server.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum ClientMessage {
    /// The first message on a new connection.
    Hello { protocol: u32 },
    /// The player takes a step on the map they are on: the server checks
    /// it, moves them and sends what comes into sight, then (once a tick)
    /// `Position`. `map` counts the maps the client has been put on
    /// (`EnterLevel`, `OnWorldMap`) since `Welcome`, so a step meant for the
    /// last map is ignored on the next; `number` counts the client's steps.
    Step { map: u32, number: u32, step: Step },
    /// Go into the site of the feature in this world map chunk (the player
    /// must be at it).
    EnterSite { chunk: ChunkPos },
    /// Use what the player stands on: stairs, or outside ground at the edge
    /// of the level (back to the world map).
    UseExit,
    /// Developer tools: jump to a point on the world map (answered with
    /// `OnWorldMap`). Only an embedded (single-player) server accepts it.
    DevTeleport { x: i64, y: i64 },
    /// Developer tools: go straight into a site of any kind and seed, at
    /// any level. Only an embedded (single-player) server accepts it.
    DevEnterSite {
        kind: FeatureKind,
        seed: u64,
        level: u32,
    },
}

/// From the server to a client.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum ServerMessage {
    /// The answer to `Hello`: the world the player is in, and where they
    /// are on its world map (where they left off). If they were in a site,
    /// `EnterLevel` follows.
    Welcome { world: WorldInfo, x: i64, y: i64 },
    /// The server won't talk to this client, and why.
    Refused(String),
    /// A world map chunk, as this player knows it.
    WorldChunk(WorldChunk),
    /// The player is now on this site level; its cells follow.
    EnterLevel(LevelInfo),
    /// Cells of the current level.
    LevelCells(LevelChunk),
    /// The player is on the world map, here: back out of a site, or moved
    /// by dev tools.
    OnWorldMap { x: i64, y: i64 },
    /// Where the server has the player on map `map` (see
    /// `ClientMessage::Step`) once it has dealt with step `step`: the client
    /// adds the steps it took since, and moves the player there if it shows
    /// them elsewhere (a step the server refused). Sent once a tick, when
    /// steps have arrived.
    Position { map: u32, step: u32, x: i64, y: i64 },
}

/// What a player knows of one world map chunk. The terrain itself isn't
/// sent: the client generates it from the terrain seed. Later this also
/// carries the chunk's changes and per-player state (visited, attitude).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct WorldChunk {
    pub chunk: ChunkPos,
    /// Its feature, if it has one.
    pub feature: Option<SeenFeature>,
}

/// A world map feature as a player knows it: what and where it is, not
/// the seed its site comes from.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct SeenFeature {
    pub kind: FeatureKind,
    pub chunk: ChunkPos,
    pub x: i64,
    pub y: i64,
}

impl From<Feature> for SeenFeature {
    fn from(feature: Feature) -> Self {
        Self {
            kind: feature.kind,
            chunk: feature.chunk,
            x: feature.x,
            y: feature.y,
        }
    }
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
    use crate::dungeon::Cell;
    use crate::site::StairsDirection;

    /// One of every message, so a message that can't be sent fails here.
    fn every_message() -> (Vec<ClientMessage>, Vec<ServerMessage>) {
        let client = vec![
            ClientMessage::Hello {
                protocol: PROTOCOL_VERSION,
            },
            ClientMessage::Step {
                map: 2,
                number: 9,
                step: Step::new(-1, 1).unwrap(),
            },
            ClientMessage::EnterSite {
                chunk: ChunkPos { x: 2, y: -1 },
            },
            ClientMessage::UseExit,
            ClientMessage::DevTeleport { x: -3, y: 4 },
            ClientMessage::DevEnterSite {
                kind: FeatureKind::Crypt,
                seed: 3,
                level: 1,
            },
        ];
        let server = vec![
            ServerMessage::Welcome {
                world: WorldInfo {
                    id: WorldId(7),
                    terrain_seed: Seed::new(8),
                    world_map_version: 1,
                },
                x: -4,
                y: 9,
            },
            ServerMessage::Refused("no".into()),
            ServerMessage::WorldChunk(WorldChunk {
                chunk: ChunkPos { x: 1, y: -2 },
                feature: Some(SeenFeature {
                    kind: FeatureKind::Castle,
                    chunk: ChunkPos { x: 1, y: -2 },
                    x: 70,
                    y: -100,
                }),
            }),
            ServerMessage::WorldChunk(WorldChunk {
                chunk: ChunkPos { x: 0, y: 0 },
                feature: None,
            }),
            ServerMessage::EnterLevel(LevelInfo {
                kind: FeatureKind::RockCave,
                building: false,
                index: 1,
                levels: 3,
                bounds: (-20, -10, 40, 30),
                arrival: (4, 5),
            }),
            ServerMessage::LevelCells(LevelChunk {
                chunk: (-1, 2),
                mask: 0b101,
                cells: vec![Cell::Floor, Cell::Door],
                stairs: vec![SeenStairs {
                    x: -6,
                    y: 16,
                    direction: StairsDirection::Down,
                    descends: (0, 1),
                }],
                remembered: true,
            }),
            ServerMessage::OnWorldMap { x: 100, y: -4 },
            ServerMessage::Position {
                map: 1,
                step: 30,
                x: -2,
                y: 5,
            },
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
