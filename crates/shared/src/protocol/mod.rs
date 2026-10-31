//! Messages between the client and the server, and the `Link` that carries
//! them. See `docs/context/shared/protocol.md`.
//!
//! Clients send intents (`ClientMessage`); the server answers with what the
//! player may know (`ServerMessage`). Every message is encoded with postcard,
//! also in single player, where the link is an in-process channel.

mod level;
mod link;

pub use level::{ENTER_REACH, LEVEL_CHUNK_SIZE, LevelChunk, LevelInfo, SeenStairs, level_chunk};
pub use link::{ClientLink, Link, LinkError, ServerLink, Wire, local_pair, wired};

use serde::{Deserialize, Serialize};

use crate::dungeon::Cell;
use crate::movement::Step;
use crate::poi::{CHUNK_SIZE, ChunkPos, Poi, PoiKind};
use crate::rng::Seed;
use crate::world::WorldId;

/// Bump whenever a message changes, so mismatched clients and servers refuse
/// each other instead of misreading messages.
pub const PROTOCOL_VERSION: u32 = 11;

/// Where a server's WebSocket is, beside the web client's files: what
/// network clients connect to.
pub const SOCKET_PATH: &str = "/ws";

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
    /// The first message on a new connection. `guest` names the character
    /// on servers without accounts (yet): a random number the client keeps,
    /// so it gets the same character back. The embedded server ignores it.
    Hello { protocol: u32, guest: u64 },
    /// Asks the server's clock, now and then. `sent` is the client's own time,
    /// handed back in `Pong` so it can tell how long the round trip took.
    Ping { sent: f64 },
    /// The player takes a step on the map they are on: the server checks
    /// it, moves them and sends what comes into sight, then (once a tick)
    /// `Position`. `map` counts the maps the client has been put on
    /// (`EnterLevel`, `OnWorldMap`) since `Welcome`, so a step meant for the
    /// last map is ignored on the next; `number` counts the client's steps.
    Step { map: u32, number: u32, step: Step },
    /// Go into the site of the poi in this world map chunk (the player
    /// must be at it).
    EnterSite { chunk: ChunkPos },
    /// Use what the player stands on: stairs, or outside ground at the edge
    /// of the level (back to the world map).
    UseExit,
    /// Developer tools: jump to a point on the map the player is on (in a
    /// site, only onto a walkable cell), answered with `Teleported`. Only an
    /// embedded (single-player) server accepts it; others ignore it.
    DevTeleport { x: i64, y: i64 },
    /// Developer tools: back to where new characters start on the world map,
    /// from anywhere (out of a site too), answered with `OnWorldMap`. Only
    /// an embedded server accepts it; others ignore it.
    DevBackToStart,
    /// Developer tools: go straight into a site of any kind and seed, at
    /// any level. Only an embedded (single-player) server accepts it.
    DevEnterSite {
        kind: PoiKind,
        seed: u64,
        level: u32,
    },
    /// Developer tools: change a cell of the level the player is on (not the
    /// one they stand on). Only an embedded server accepts it.
    DevSetCell { x: i64, y: i64, cell: Cell },
    /// Developer tools: set the snow line (which freezes water), answered
    /// with `SnowLine`. Only an embedded server accepts it.
    DevSnowLine(u32),
}

impl ClientMessage {
    /// Whether it is a developer tools message (`Dev...`), which only an
    /// embedded server accepts.
    pub fn is_dev_tool(&self) -> bool {
        matches!(
            self,
            Self::DevTeleport { .. }
                | Self::DevBackToStart
                | Self::DevEnterSite { .. }
                | Self::DevSetCell { .. }
                | Self::DevSnowLine(_)
        )
    }
}

/// From the server to a client.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum ServerMessage {
    /// The answer to `Hello`: the world the player is in, its clock (in game
    /// hours since it began) and snow line, and where they are on its world
    /// map (where they left off). If they were in a site, `EnterLevel` follows.
    Welcome {
        world: WorldInfo,
        clock: f64,
        snow_line: u32,
        x: i64,
        y: i64,
    },
    /// The snow line has moved: above it everything is frozen, and ice can be
    /// walked on.
    SnowLine(u32),
    /// A new whole hour has begun on the clock (`% 24` is the time of day).
    /// The tide, and so where can be walked on the world map, follows it.
    Hour(u32),
    /// The answer to `Ping`: its `sent`, and the world's clock when answering.
    Pong { sent: f64, clock: f64 },
    /// The server won't talk to this client, and why.
    Refused(String),
    /// A world map chunk, as this player knows it.
    WorldChunk(WorldChunk),
    /// The player is now on this site level; its cells follow.
    EnterLevel(LevelInfo),
    /// Cells of the current level.
    LevelCells(LevelChunk),
    /// A town's buildings, sent after `EnterLevel` and before its cells:
    /// players know a town's layout from the start.
    Buildings(Vec<crate::site::BuildingLook>),
    /// The player is on the world map, here: back out of a site, or sent
    /// back to the start by dev tools.
    OnWorldMap { x: i64, y: i64 },
    /// Developer tools: the player has jumped to here, on the map they are
    /// on (`DevTeleport`).
    Teleported { x: i64, y: i64 },
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
    /// Its poi, if it has one.
    pub poi: Option<SeenPoi>,
}

/// A world map poi as a player knows it: what and where it is, not
/// the seed its site comes from.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct SeenPoi {
    pub kind: PoiKind,
    pub chunk: ChunkPos,
    pub x: i64,
    pub y: i64,
}

impl From<Poi> for SeenPoi {
    fn from(poi: Poi) -> Self {
        Self {
            kind: poi.kind,
            chunk: poi.chunk,
            x: poi.x,
            y: poi.y,
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
    use crate::site::{StairsDirection, StairsStyle};

    /// One of every message, so a message that can't be sent fails here.
    fn every_message() -> (Vec<ClientMessage>, Vec<ServerMessage>) {
        let client = vec![
            ClientMessage::Hello {
                protocol: PROTOCOL_VERSION,
                guest: 0x1234_5678_9abc_def0,
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
            ClientMessage::DevSnowLine(40),
            ClientMessage::DevBackToStart,
            ClientMessage::DevEnterSite {
                kind: PoiKind::Crypt,
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
                clock: 5.25,
                snow_line: 128,
                x: -4,
                y: 9,
            },
            ServerMessage::SnowLine(40),
            ServerMessage::Hour(6),
            ServerMessage::Pong {
                sent: 1.5,
                clock: 6.75,
            },
            ServerMessage::Refused("no".into()),
            ServerMessage::WorldChunk(WorldChunk {
                chunk: ChunkPos { x: 1, y: -2 },
                poi: Some(SeenPoi {
                    kind: PoiKind::Castle,
                    chunk: ChunkPos { x: 1, y: -2 },
                    x: 70,
                    y: -100,
                }),
            }),
            ServerMessage::WorldChunk(WorldChunk {
                chunk: ChunkPos { x: 0, y: 0 },
                poi: None,
            }),
            ServerMessage::EnterLevel(LevelInfo {
                kind: PoiKind::RockCave,
                building: false,
                index: 1,
                levels: 3,
                bounds: (-20, -10, 40, 30),
                arrival: (4, 5),
            }),
            ServerMessage::Buildings(vec![crate::site::BuildingLook {
                id: 3,
                kind: crate::site::BuildingKind::Bank,
                walls: crate::site::WallKind::Brick,
                roof: crate::site::RoofKind::SlateTiles,
                rects: vec![crate::site::Rect {
                    x: 1,
                    y: -2,
                    width: 3,
                    height: 4,
                }],
                door: (2, -2),
                faces: (0, -1),
            }]),
            ServerMessage::LevelCells(LevelChunk {
                chunk: (-1, 2),
                mask: 0b101,
                cells: vec![Cell::Floor, Cell::Door],
                stairs: vec![SeenStairs {
                    x: -6,
                    y: 16,
                    direction: StairsDirection::Down,
                    descends: (0, 1),
                    style: StairsStyle::Spiral,
                }],
                remembered: true,
            }),
            ServerMessage::OnWorldMap { x: 100, y: -4 },
            ServerMessage::Teleported { x: 6, y: -7 },
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
        assert!(!client.send(&ClientMessage::UseExit));
    }

    #[test]
    fn wires_carry_the_bytes_both_ways() {
        // Two wired links joined by their wires, as over a socket.
        let (client, client_wire) = wired::<ClientMessage, ServerMessage>();
        let (server, server_wire) = wired::<ServerMessage, ClientMessage>();
        let (to_server, to_client) = every_message();
        for message in &to_server {
            assert!(client.send(message));
            let bytes = client_wire.take().unwrap().unwrap();
            assert!(server_wire.deliver(bytes));
            assert_eq!(server.receive(), Ok(Some(message.clone())));
        }
        for message in &to_client {
            assert!(server.send(message));
            let bytes = server_wire.take().unwrap().unwrap();
            assert!(client_wire.deliver(bytes));
            assert_eq!(client.receive(), Ok(Some(message.clone())));
        }
        // Garbage from the socket is noticed, not misread.
        assert!(server_wire.deliver(vec![0xff, 0xff, 0xff]));
        assert_eq!(server.receive(), Err(LinkError::Garbled));
        // Either side going is noticed by the other.
        drop(client);
        assert_eq!(client_wire.take(), Err(LinkError::Closed));
        drop(server_wire);
        assert_eq!(server.receive(), Err(LinkError::Closed));
    }
}
