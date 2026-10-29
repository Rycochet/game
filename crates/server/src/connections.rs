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
    /// How many maps the player has been put on since `Welcome` (see
    /// `ClientMessage::Step`).
    map: u32,
    /// The last step dealt with this tick, to send `Position` for.
    last_step: Option<u32>,
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
            map: 0,
            last_step: None,
        }
    }

    /// Where the player is on the world map (at the feature, in a site).
    pub(crate) fn world_position(&self) -> (i64, i64) {
        self.character.world_position
    }

    /// The player is being put on another map (the message saying so is
    /// sent next): steps the client took on the last one no longer count.
    pub(crate) fn changed_map(&mut self) {
        self.map += 1;
        self.last_step = None;
    }

    /// If steps arrived this tick, tells the client where the player is
    /// after the last of them. `false` if the client has gone.
    fn send_position(&mut self) -> bool {
        let Some(step) = self.last_step.take() else {
            return true;
        };
        let (x, y) = match &self.visit {
            Some(visit) => visit.position(),
            None => self.world_position(),
        };
        let map = self.map;
        self.link.send(&ServerMessage::Position { map, step, x, y })
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
                Ok(None) => break client.send_position(),
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
        // Meant for a map the player has since left.
        ClientMessage::Step { map, .. } if map != client.map => true,
        ClientMessage::Step { number, step, .. } => {
            client.last_step = Some(number);
            match client.visit {
                Some(_) => sites::stepped(client, step),
                None => world_map::stepped(client, world, step),
            }
        }
        ClientMessage::EnterSite { chunk } => sites::enter_feature(client, world, chunk),
        ClientMessage::UseExit => sites::use_exit(client, world),
        ClientMessage::DevTeleport { x, y } if options.dev_tools => {
            world_map::teleport(client, world, x, y)
        }
        ClientMessage::DevEnterSite { kind, seed, level } if options.dev_tools => {
            sites::dev_enter(client, world, kind, seed, level)
        }
        ClientMessage::DevEnterSite { .. } | ClientMessage::DevTeleport { .. } => {
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
        None => world_map::look_around(client, world),
    }
}

#[cfg(test)]
pub(crate) mod tests {
    use shared::movement::{Step, steps_between};
    use shared::protocol::{ClientLink, local_pair};
    use shared::world::WorldSeed;

    use super::*;

    /// A server world, a client connected to it, and the client's end.
    pub(crate) struct TestClient {
        pub world: ServerWorld,
        pub client: Client,
        pub end: ClientLink,
        /// The answers to `Hello`.
        pub welcome: Vec<ServerMessage>,
        next_step: u32,
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
                welcome: Vec::new(),
                next_step: 0,
            };
            test.welcome = test.send(&[HELLO]);
            test
        }

        /// Sends messages (in one tick), and returns everything answered
        /// since last time.
        pub fn send(&mut self, messages: &[ClientMessage]) -> Vec<ServerMessage> {
            let options = ServerOptions {
                dev_tools: true,
                exit_when_alone: false,
            };
            for message in messages {
                answer(&mut self.client, message.clone(), &mut self.world, &options);
            }
            self.client.send_position();
            std::iter::from_fn(|| self.end.receive().unwrap()).collect()
        }

        /// Takes steps (`(dx, dy)`) on the current map, numbered on from
        /// the last ones, and returns the answers.
        pub fn steps(&mut self, steps: &[(i64, i64)]) -> Vec<ServerMessage> {
            let messages: Vec<_> = steps
                .iter()
                .map(|&(dx, dy)| {
                    self.next_step += 1;
                    ClientMessage::Step {
                        map: self.client.map,
                        number: self.next_step,
                        step: Step::new(dx, dy).expect("a step"),
                    }
                })
                .collect();
            self.send(&messages)
        }

        /// How many maps the player has been put on (`Step::map`).
        pub fn client_map(&self) -> u32 {
            self.client.map
        }

        /// Walks along the steps from where the player is to `to`.
        pub fn walk_to(&mut self, to: (i64, i64)) -> Vec<ServerMessage> {
            let from = match &self.client.visit {
                Some(visit) => visit.position(),
                None => self.client.world_position(),
            };
            let steps: Vec<_> = steps_between(from, to)
                .into_iter()
                .map(|s| (s.dx(), s.dy()))
                .collect();
            self.steps(&steps)
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
        let message = ClientMessage::UseExit;
        assert!(!answer(&mut client, message, &mut world, &options));
        assert!(matches!(end.receive(), Ok(Some(ServerMessage::Refused(_)))));
    }

    #[test]
    fn steps_are_answered_with_the_position_once_a_tick() {
        let mut test = TestClient::new();
        let answers = test.steps(&[(1, 0), (1, 1), (0, 1)]);
        let positions: Vec<_> = answers
            .iter()
            .filter(|a| matches!(a, ServerMessage::Position { .. }))
            .collect();
        assert_eq!(
            positions,
            [&ServerMessage::Position {
                map: 0,
                step: 3,
                x: 2,
                y: 2
            }]
        );
        // Nothing moved, nothing to say.
        assert!(test.send(&[]).is_empty());
    }

    #[test]
    fn steps_meant_for_another_map_are_ignored() {
        let mut test = TestClient::new();
        let stale = ClientMessage::Step {
            map: 7,
            number: 1,
            step: Step::new(1, 0).unwrap(),
        };
        assert!(test.send(&[stale]).is_empty());
        assert_eq!(test.client.world_position(), (0, 0));
    }
}
