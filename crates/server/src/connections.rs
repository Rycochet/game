//! Connected clients: reading their messages, answering them, and saving
//! their characters.

use std::collections::BTreeSet;

use bevy::prelude::*;
use shared::feature::ChunkPos;
use shared::protocol::{ClientMessage, PROTOCOL_VERSION, ServerLink, ServerMessage};

use crate::sites::{self, Visit};
use crate::storage::{CharacterName, CharacterSave};
use crate::{ServerOptions, ServerWorld, world_map};

/// The clients connected to this server.
#[derive(Resource, Default)]
pub struct Connections(Vec<Client>);

impl Connections {
    /// A newly connected client, playing the character `name` (decided by
    /// whoever accepted the connection: the embedded server's one player,
    /// later a logged-in account's character).
    pub fn add(&mut self, link: ServerLink, name: CharacterName) {
        self.0.push(Client::new(link, name));
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}

/// One connected client: its character, where it is and what it has been
/// sent.
pub struct Client {
    pub(crate) link: ServerLink,
    name: CharacterName,
    /// Whether it has said `Hello` (and been welcomed, with its character
    /// loaded).
    welcomed: bool,
    /// What is saved about the character: where they are on the world map,
    /// the features and cells they know (`visit` is filled in on saving).
    pub(crate) character: CharacterSave,
    /// Whether `character` (or the visit) changed since it was saved.
    pub(crate) unsaved: bool,
    /// The site level the player is on, if any.
    pub(crate) visit: Option<Visit>,
    /// The world map chunks it has been sent on this connection.
    pub(crate) world_chunks: BTreeSet<ChunkPos>,
}

impl Client {
    pub(crate) fn new(link: ServerLink, name: CharacterName) -> Self {
        Self {
            link,
            name,
            welcomed: false,
            character: CharacterSave::default(),
            unsaved: false,
            visit: None,
            world_chunks: BTreeSet::new(),
        }
    }

    /// Where the player is on the world map (at the feature, in a site).
    pub(crate) fn world_position(&self) -> (i64, i64) {
        self.character.world_position
    }

    /// Saves the character if anything changed (only once welcomed: before
    /// that nothing has been loaded).
    fn save(&mut self, world: &mut ServerWorld) {
        if !self.welcomed || !self.unsaved {
            return;
        }
        self.character.visit = self.visit.as_ref().map(Visit::save);
        world.storage.save_character(&self.name, &self.character);
        self.unsaved = false;
    }
}

/// Answers every message that has arrived, and forgets clients that have
/// gone (or were refused), saving them first.
pub(crate) fn handle_messages(
    mut connections: ResMut<Connections>,
    mut world: ResMut<ServerWorld>,
    options: Res<ServerOptions>,
) {
    connections.0.retain_mut(|client| {
        let stay = loop {
            match client.link.receive() {
                Ok(Some(message)) => {
                    if !answer(client, message, &mut world, &options) {
                        break false;
                    }
                }
                Ok(None) => break true,
                Err(error) => {
                    info!("A client left ({error:?})");
                    break false;
                }
            }
        };
        if !stay {
            sites::leave(client, &mut world);
            client.save(&mut world);
            world.save_sites();
        }
        stay
    });
}

/// Saves every character and site that has changed.
pub(crate) fn save_changes(mut connections: ResMut<Connections>, mut world: ResMut<ServerWorld>) {
    for client in &mut connections.0 {
        client.save(&mut world);
    }
    world.save_sites();
}

/// Answers one message; `false` if the client should be disconnected.
pub(crate) fn answer(
    client: &mut Client,
    message: ClientMessage,
    world: &mut ServerWorld,
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
        ClientMessage::Hello { .. } if client.welcomed => refuse(client, "hello again?".into()),
        ClientMessage::Hello { .. } => welcome(client, world),
        _ if !client.welcomed => refuse(client, "say hello first".into()),
        ClientMessage::WorldMapPosition { x, y } => world_map::moved_to(client, world, x, y),
        ClientMessage::EnterSite { chunk } => sites::enter_feature(client, world, chunk),
        ClientMessage::SitePosition { x, y } => sites::moved_to(client, x, y),
        ClientMessage::UseExit => sites::use_exit(client, world),
        ClientMessage::DevEnterSite { kind, seed, level } if options.dev_tools => {
            sites::dev_enter(client, world, kind, seed, level)
        }
        ClientMessage::DevEnterSite { .. } => {
            refuse(client, "this server doesn't allow dev tools".into())
        }
    }
}

/// Loads the character (or starts a new one), and tells the client the
/// world and where they are; back into the site level they were on, if any.
fn welcome(client: &mut Client, world: &mut ServerWorld) -> bool {
    client.welcomed = true;
    client.character = world
        .storage
        .load_character(&client.name)
        .unwrap_or_default();
    let (x, y) = client.world_position();
    let welcome = ServerMessage::Welcome {
        world: world.info(),
        x,
        y,
    };
    if !client.link.send(&welcome) {
        return false;
    }
    match client.character.visit.take() {
        Some(visit) => sites::resume(client, world, visit),
        None => true,
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
        /// A client of a world that isn't saved, welcomed.
        pub fn new() -> Self {
            Self::of(ServerWorld::new(WorldSeed::new(1)))
        }

        /// A client of `world`, welcomed (its answer to `Hello` is read).
        pub fn of(world: ServerWorld) -> Self {
            let (end, server_link) = local_pair();
            let mut test = Self {
                world,
                client: Client::new(server_link, CharacterName::new("tester").unwrap()),
                end,
            };
            test.send(&[HELLO]);
            test
        }

        /// Sends messages, and returns everything answered since last time.
        pub fn send(&mut self, messages: &[ClientMessage]) -> Vec<ServerMessage> {
            let options = ServerOptions {
                dev_tools: true,
                exit_when_alone: false,
            };
            for message in messages {
                answer(&mut self.client, message.clone(), &mut self.world, &options);
            }
            std::iter::from_fn(|| self.end.receive().unwrap()).collect()
        }

        /// The client leaves: saves it (as `handle_messages` does), and
        /// returns the world.
        pub fn leave(mut self) -> ServerWorld {
            sites::leave(&self.client, &mut self.world);
            self.client.save(&mut self.world);
            self.world.save_sites();
            self.world
        }
    }

    pub(crate) const HELLO: ClientMessage = ClientMessage::Hello {
        protocol: PROTOCOL_VERSION,
    };

    #[test]
    fn messages_before_hello_are_refused() {
        let (end, server_link) = local_pair();
        let mut client = Client::new(server_link, CharacterName::new("x").unwrap());
        let mut world = ServerWorld::new(WorldSeed::new(1));
        let options = ServerOptions {
            dev_tools: false,
            exit_when_alone: false,
        };
        let message = ClientMessage::WorldMapPosition { x: 0, y: 0 };
        assert!(!answer(&mut client, message, &mut world, &options));
        assert!(matches!(end.receive(), Ok(Some(ServerMessage::Refused(_)))));
    }
}
