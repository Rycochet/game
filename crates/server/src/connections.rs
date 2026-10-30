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
    /// A newly connected client, playing the character `name` if given
    /// (decided by whoever accepted the connection: the embedded server's
    /// one player, later a logged-in account's character); otherwise the
    /// guest its `Hello` names.
    pub fn add(&mut self, link: ServerLink, name: Option<CharacterName>) {
        self.0.push(Client::new(link, name));
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    /// Disconnects everyone, saving them (the server is stopping).
    pub(crate) fn disconnect_all(&mut self, world: &mut ServerWorld) {
        for client in &mut self.0 {
            client.disconnect(world);
        }
        self.0.clear();
    }
}

/// One connected client: its character, where it is and what it has been
/// sent.
pub struct Client {
    pub(crate) link: ServerLink,
    /// Its character: given when it connected, or from its `Hello`.
    name: Option<CharacterName>,
    /// Whether it has said `Hello` and is waiting to be welcomed.
    joining: bool,
    /// Whether it has been welcomed (with its character loaded).
    welcomed: bool,
    /// Whether it is to be disconnected (and was saved).
    gone: bool,
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
    pub(crate) fn new(link: ServerLink, name: Option<CharacterName>) -> Self {
        Self {
            link,
            name,
            joining: false,
            welcomed: false,
            gone: false,
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
        let Some(name) = &self.name else {
            return;
        };
        if !self.welcomed || !self.unsaved {
            return;
        }
        self.character.visit = self.visit.as_ref().map(Visit::save);
        world.storage.save_character(name, &self.character);
        self.unsaved = false;
    }

    /// Disconnects it: the player leaves their level, and the character is
    /// saved.
    fn disconnect(&mut self, world: &mut ServerWorld) {
        self.gone = true;
        sites::leave(self, world);
        self.save(world);
        world.save_sites();
    }
}

/// Answers every message that has arrived, and forgets clients that have
/// gone (or were refused), saving them first.
pub(crate) fn handle_messages(
    mut connections: ResMut<Connections>,
    mut world: ResMut<ServerWorld>,
    options: Res<ServerOptions>,
) {
    let clients = &mut connections.0;
    for i in 0..clients.len() {
        let client = &mut clients[i];
        if client.gone {
            continue;
        }
        let stay = loop {
            match client.link.receive() {
                Ok(Some(message)) => {
                    if !answer(client, message, &mut world, &options) {
                        break false;
                    }
                    // The rest once it has been welcomed (below).
                    if client.joining {
                        break true;
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
            client.disconnect(&mut world);
        } else if client.joining && !join(clients, i, &mut world) {
            clients[i].disconnect(&mut world);
        }
    }
    clients.retain(|client| !client.gone);
}

/// Welcomes client `i`, which has said hello. Anyone else playing its
/// character is disconnected first (and saved, so the character loads as
/// they left it): the newest connection wins, for coming back after a
/// connection drops before the server has noticed. `false` if the client
/// has gone.
fn join(clients: &mut [Client], i: usize, world: &mut ServerWorld) -> bool {
    let name = clients[i].name.clone();
    for (j, other) in clients.iter_mut().enumerate() {
        if j != i && !other.gone && other.welcomed && other.name == name {
            info!("{name:?} connected again: dropping the older connection");
            other.link.send(&ServerMessage::Refused(
                "this character is playing somewhere else now".into(),
            ));
            other.disconnect(world);
        }
    }
    welcome(&mut clients[i], world)
}

/// Saves every character and site that has changed.
pub(crate) fn save_changes(mut connections: ResMut<Connections>, mut world: ResMut<ServerWorld>) {
    for client in &mut connections.0 {
        client.save(&mut world);
    }
    world.save_sites();
}

/// A new game hour: saved, and told to everyone playing (the tide follows it).
/// A client that has gone is noticed when its messages are next read.
pub(crate) fn next_hour(connections: Res<Connections>, mut world: ResMut<ServerWorld>) {
    world.hour = (world.hour + 1) % 24;
    world.save();
    for client in connections.0.iter().filter(|client| client.welcomed) {
        client.link.send(&ServerMessage::Hour(world.hour));
    }
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
        ClientMessage::Hello { protocol, .. } if protocol != PROTOCOL_VERSION => refuse(
            client,
            format!(
                "this server speaks protocol {PROTOCOL_VERSION}, not {protocol}: \
                 the game needs updating (reload the page)"
            ),
        ),
        ClientMessage::Hello { .. } if client.welcomed || client.joining => {
            refuse(client, "hello again?".into())
        }
        ClientMessage::Hello { guest, .. } => {
            if client.name.is_none() {
                client.name = CharacterName::new(&format!("guest-{guest:016x}"));
            }
            // Welcomed by `handle_messages`, which can see other clients.
            client.joining = true;
            true
        }
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
        // Dev tools are only for the embedded server; elsewhere a stray key
        // or click is ignored rather than ending the game.
        _ if message.is_dev_tool() && !options.dev_tools => {
            warn!("A client asked for dev tools ({message:?}), which this server doesn't allow");
            true
        }
        ClientMessage::DevTeleport { x, y } => match client.visit {
            Some(_) => sites::teleport(client, x, y),
            None => world_map::teleport(client, world, x, y),
        },
        ClientMessage::DevBackToStart => world_map::back_to_start(client, world),
        ClientMessage::DevEnterSite { kind, seed, level } => {
            sites::dev_enter(client, world, kind, seed, level)
        }
        ClientMessage::DevSetCell { x, y, cell } => sites::set_cell(client, world, x, y, cell),
        ClientMessage::DevSnowLine(snow_line) => {
            world.snow_line = snow_line;
            client.link.send(&ServerMessage::SnowLine(snow_line))
        }
    }
}

/// Loads the character (or starts a new one), and tells the client the
/// world and where they are; back into the site level they were on, if any.
fn welcome(client: &mut Client, world: &mut ServerWorld) -> bool {
    client.joining = false;
    client.welcomed = true;
    let name = client.name.as_ref().expect("named on hello");
    client.character = world.storage.load_character(name).unwrap_or_default();
    let (x, y) = client.world_position();
    let welcome = ServerMessage::Welcome {
        world: world.info(),
        hour: world.hour,
        snow_line: world.snow_line,
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
        /// Whether the server allows dev tools (it does, unless set).
        pub dev_tools: bool,
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
                client: Client::new(server_link, CharacterName::new("tester")),
                end,
                welcome: Vec::new(),
                dev_tools: true,
                next_step: 0,
            };
            test.welcome = test.send(&[HELLO]);
            test
        }

        /// Sends messages (in one tick), and returns everything answered
        /// since last time.
        pub fn send(&mut self, messages: &[ClientMessage]) -> Vec<ServerMessage> {
            let options = ServerOptions {
                dev_tools: self.dev_tools,
                exit_when_alone: false,
            };
            for message in messages {
                answer(&mut self.client, message.clone(), &mut self.world, &options);
                if self.client.joining {
                    welcome(&mut self.client, &mut self.world);
                }
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
        guest: 1,
    };

    /// Runs `handle_messages` once over these connections.
    fn tick(connections: Connections, world: ServerWorld) -> (Connections, ServerWorld) {
        use bevy::ecs::system::RunSystemOnce;
        let mut app = World::new();
        app.insert_resource(connections);
        app.insert_resource(world);
        app.insert_resource(ServerOptions {
            dev_tools: false,
            exit_when_alone: false,
        });
        app.run_system_once(handle_messages).unwrap();
        (
            app.remove_resource::<Connections>().unwrap(),
            app.remove_resource::<ServerWorld>().unwrap(),
        )
    }

    fn answers(end: &ClientLink) -> Vec<ServerMessage> {
        std::iter::from_fn(|| end.receive().ok().flatten()).collect()
    }

    #[test]
    fn guests_are_named_by_their_hello_and_the_newest_connection_wins() {
        let folder = crate::storage::tests::TestFolder::new();
        let seed = WorldSeed::new(1);
        let storage = crate::storage::Storage::in_folder(&folder.0, seed.id());
        let world = ServerWorld::open(seed, storage).unwrap();
        let mut connections = Connections::default();
        let (first, link) = local_pair();
        connections.add(link, None);
        first.send(&HELLO);
        let step = |n| ClientMessage::Step {
            map: 0,
            number: n,
            step: Step::new(1, 0).unwrap(),
        };
        let (connections, world) = tick(connections, world);
        assert!(matches!(
            answers(&first)[0],
            ServerMessage::Welcome { x: 0, .. }
        ));
        first.send(&step(1));
        let (mut connections, world) = tick(connections, world);
        answers(&first);

        // The same guest again (a reconnect): the old connection goes, and
        // the character comes back where it was.
        let (second, link) = local_pair();
        connections.add(link, None);
        second.send(&HELLO);
        let (connections, _world) = tick(connections, world);
        assert!(matches!(answers(&first)[..], [ServerMessage::Refused(_)]));
        assert!(matches!(
            answers(&second)[0],
            ServerMessage::Welcome { x: 1, .. }
        ));
        assert_eq!(connections.0.len(), 1);
    }

    #[test]
    fn every_hour_is_told_saved_and_wraps() {
        use bevy::ecs::system::RunSystemOnce;
        let folder = crate::storage::tests::TestFolder::new();
        let seed = WorldSeed::new(1);
        let open = || {
            let storage = crate::storage::Storage::in_folder(&folder.0, seed.id());
            ServerWorld::open(seed, storage).unwrap()
        };
        let mut connections = Connections::default();
        let (end, link) = local_pair();
        connections.add(link, None);
        end.send(&HELLO);
        let (connections, mut world) = tick(connections, open());
        answers(&end);

        world.hour = 23;
        let mut app = World::new();
        app.insert_resource(connections);
        app.insert_resource(world);
        app.run_system_once(next_hour).unwrap();
        assert_eq!(answers(&end), [ServerMessage::Hour(0)]);
        app.run_system_once(next_hour).unwrap();
        assert_eq!(answers(&end), [ServerMessage::Hour(1)]);
        // A restart carries on from there, and new players are told.
        assert_eq!(open().hour, 1);
    }

    #[test]
    fn messages_before_hello_are_refused() {
        let (end, server_link) = local_pair();
        let mut client = Client::new(server_link, CharacterName::new("x"));
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
