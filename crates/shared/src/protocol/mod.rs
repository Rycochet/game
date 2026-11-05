//! Messages between the client and the server, and the `Link` that carries
//! them. See `docs/context/shared/protocol.md`.
//!
//! Clients send intents (`ClientMessage`); the server answers with what the
//! player may know (`ServerMessage`). Every message is encoded with postcard,
//! also in single player, where the link is an in-process channel.

mod address;
mod level;
mod link;
mod login;
mod online;
mod version;

pub use address::{DEFAULT_PORT, FALLBACK_PORT, ServerAddress};
pub use level::{
    ENTER_REACH, LEVEL_CHUNK_SIZE, LevelChunk, LevelInfo, LevelKind, RememberedLevel, SeenStairs,
    level_chunk,
};
pub use link::{ClientLink, Link, LinkError, ServerLink, Wire, local_pair, wired};
pub use login::{CharacterId, CharacterSummary, Credentials, GuestToken, LoginMethod};
pub use online::{OnlineSettings, OnlineState};
pub use version::{GAME_NAME, ServerVersion, VERSION_PATH};

use crate::poi::CHUNK_SIZE;

// Split into files (`cargo xtask lint` keeps files small).
mod client_message;
mod ping;
mod server_message;
mod world_chunk;

pub use client_message::ClientMessage;
pub use server_message::ServerMessage;
pub use world_chunk::{SeenPoi, WorldChunk, WorldInfo};

/// Bump whenever a message changes, so mismatched clients and servers refuse
/// each other instead of misreading messages.
pub const PROTOCOL_VERSION: u32 = 42;

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

#[cfg(test)]
mod tests;
