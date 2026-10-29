//! Connected clients: reading their messages and answering them.

use std::collections::{BTreeMap, BTreeSet};

use bevy::prelude::*;
use shared::feature::ChunkPos;
use shared::protocol::{ClientMessage, PROTOCOL_VERSION, ServerLink, ServerMessage};

use crate::sites::{self, CellMasks, LevelKey, Visit};
use crate::{ServerOptions, ServerWorld, world_map};

/// The clients connected to this server.
#[derive(Resource, Default)]
pub struct Connections(Vec<Client>);

impl Connections {
    /// A newly connected client.
    pub fn add(&mut self, link: ServerLink) {
        self.0.push(Client::new(link));
    }
}

/// One connected client: where its player is and what it has been sent.
pub struct Client {
    pub(crate) link: ServerLink,
    /// Whether it has said `Hello` (and been welcomed).
    welcomed: bool,
    /// The world map chunks it has been sent.
    pub(crate) world_chunks: BTreeSet<ChunkPos>,
    /// Where the player last said they were on the world map.
    pub(crate) world_position: Option<(i64, i64)>,
    /// The site level the player is on, if any.
    pub(crate) visit: Option<Visit>,
    /// The cells the player has seen of each site level. Kept for this
    /// connection only, until there is persistence.
    pub(crate) memory: BTreeMap<LevelKey, CellMasks>,
}

impl Client {
    pub(crate) fn new(link: ServerLink) -> Self {
        Self {
            link,
            welcomed: false,
            world_chunks: BTreeSet::new(),
            world_position: None,
            visit: None,
            memory: BTreeMap::new(),
        }
    }
}

/// Answers every message that has arrived, and forgets clients that have
/// gone (or were refused).
pub(crate) fn handle_messages(
    mut connections: ResMut<Connections>,
    world: Res<ServerWorld>,
    options: Res<ServerOptions>,
) {
    connections.0.retain_mut(|client| {
        loop {
            match client.link.receive() {
                Ok(Some(message)) => {
                    if !answer(client, message, &world, &options) {
                        return false;
                    }
                }
                Ok(None) => return true,
                Err(error) => {
                    info!("A client left ({error:?})");
                    return false;
                }
            }
        }
    });
}

/// Answers one message; `false` if the client should be disconnected.
pub(crate) fn answer(
    client: &mut Client,
    message: ClientMessage,
    world: &ServerWorld,
    options: &ServerOptions,
) -> bool {
    let refuse = |client: &Client, reason: String| {
        client.link.send(&ServerMessage::Refused(reason));
        false
    };
    match message {
        ClientMessage::Hello { protocol } if protocol != PROTOCOL_VERSION => refuse(
            client,
            format!("this server speaks protocol {PROTOCOL_VERSION}, not {protocol}"),
        ),
        ClientMessage::Hello { .. } => {
            client.welcomed = true;
            client.link.send(&ServerMessage::Welcome(world.info()))
        }
        _ if !client.welcomed => refuse(client, "say hello first".into()),
        ClientMessage::WorldMapPosition { x, y } => world_map::moved_to(client, world, x, y),
        ClientMessage::EnterSite { chunk } => sites::enter_feature(client, world, chunk),
        ClientMessage::SitePosition { x, y } => sites::moved_to(client, x, y),
        ClientMessage::UseExit => sites::use_exit(client),
        ClientMessage::DevEnterSite { kind, seed, level } if options.dev_tools => {
            sites::dev_enter(client, kind, seed, level)
        }
        ClientMessage::DevEnterSite { .. } => {
            refuse(client, "this server doesn't allow dev tools".into())
        }
    }
}

#[cfg(test)]
pub(crate) mod tests {
    use shared::protocol::{ClientLink, local_pair};
    use shared::world::WorldSeed;

    use super::*;

    /// A server world, a client connected to it, and the client's end.
    pub(crate) struct TestClient {
        pub world: ServerWorld,
        pub client: Client,
        pub end: ClientLink,
    }

    impl TestClient {
        pub fn new() -> Self {
            let (end, server_link) = local_pair();
            let mut test = Self {
                world: ServerWorld::new(WorldSeed::new(1)),
                client: Client::new(server_link),
                end,
            };
            test.send(&[HELLO]);
            test
        }

        /// Sends messages, and returns everything answered since last time.
        pub fn send(&mut self, messages: &[ClientMessage]) -> Vec<ServerMessage> {
            let options = ServerOptions { dev_tools: true };
            for message in messages {
                answer(&mut self.client, message.clone(), &self.world, &options);
            }
            std::iter::from_fn(|| self.end.receive().unwrap()).collect()
        }
    }

    pub(crate) const HELLO: ClientMessage = ClientMessage::Hello {
        protocol: PROTOCOL_VERSION,
    };

    #[test]
    fn messages_before_hello_are_refused() {
        let (end, server_link) = local_pair();
        let mut client = Client::new(server_link);
        let world = ServerWorld::new(WorldSeed::new(1));
        let options = ServerOptions { dev_tools: false };
        let message = ClientMessage::WorldMapPosition { x: 0, y: 0 };
        assert!(!answer(&mut client, message, &world, &options));
        assert!(matches!(end.receive(), Ok(Some(ServerMessage::Refused(_)))));
    }
}
