//! Messages between the client and the server, and the `Link` that carries
//! them. See `docs/context/shared/protocol.md`.
//!
//! Clients send intents (`ClientMessage`); the server answers with what the
//! player may know (`ServerMessage`). Every message is encoded with postcard,
//! also in single player, where the link is an in-process channel.

mod link;

pub use link::{ClientLink, Link, LinkError, ServerLink, local_pair};

use serde::{Deserialize, Serialize};

use crate::rng::Seed;
use crate::world::WorldId;

/// Bump whenever a message changes, so mismatched clients and servers refuse
/// each other instead of misreading messages.
pub const PROTOCOL_VERSION: u32 = 1;

/// From a client to the server.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum ClientMessage {
    /// The first message on a new connection.
    Hello { protocol: u32 },
}

/// From the server to a client.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum ServerMessage {
    /// The answer to `Hello`: the world the player is in.
    Welcome(WorldInfo),
    /// The server won't talk to this client, and why.
    Refused(String),
}

/// What a client is told about a world when it joins.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct WorldInfo {
    pub id: WorldId,
    /// Public: the client generates the world map terrain from it.
    pub terrain_seed: Seed,
    /// Which world map generator the world was made with (`WORLD_MAP_VERSION`).
    pub world_map_version: u32,
    /// TEMPORARY, until the server sends features: the secret features seed,
    /// so the client can still place features and generate sites itself.
    pub features_seed: Seed,
}

#[cfg(test)]
mod tests {
    use super::*;

    /// One of every message, so a message that can't be sent fails here.
    fn every_message() -> (Vec<ClientMessage>, Vec<ServerMessage>) {
        let client = vec![ClientMessage::Hello {
            protocol: PROTOCOL_VERSION,
        }];
        let server = vec![
            ServerMessage::Welcome(WorldInfo {
                id: WorldId(7),
                terrain_seed: Seed::new(8),
                world_map_version: 1,
                features_seed: Seed::new(9),
            }),
            ServerMessage::Refused("no".into()),
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
