//! Connected clients: reading their messages, answering them, and saving
//! their characters.

use std::collections::BTreeSet;

use bevy::prelude::*;
use shared::poi::ChunkPos;
use shared::protocol::{ClientMessage, PROTOCOL_VERSION, ServerLink, ServerMessage};
use shared::world::MINUTES_PER_HOUR;

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
    /// the pois and cells they know (`visit` is filled in on saving).
    pub(crate) character: CharacterSave,
    /// Whether `character` (or the visit) changed since it was saved.
    pub(crate) unsaved: bool,
    /// The site level the player is on, if any.
    pub(crate) visit: Option<Visit>,
    /// The world map chunks it has been sent on this connection.
    pub(crate) world_chunks: BTreeSet<ChunkPos>,
    /// Every poi is shown, whether known or not: from the start while
    /// `ALL_POIS_KNOWN`, otherwise by developer tools.
    pub(crate) reveal_pois: bool,
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
            reveal_pois: world_map::ALL_POIS_KNOWN,
            map: 0,
            last_step: None,
        }
    }

    /// Where the player is on the world map (at the poi, in a site).
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
        } else if client.joining && !join(clients, i, &mut world, options.dev_tools) {
            clients[i].disconnect(&mut world);
        }
        put_out_of_forgotten_sites(clients, i, &mut world);
    }
    clients.retain(|client| !client.gone);
}

/// Puts anyone but client `i` (who did the forgetting, and has been dealt with)
/// out of any site just forgotten: what they are on is no longer that site.
fn put_out_of_forgotten_sites(clients: &mut [Client], i: usize, world: &mut ServerWorld) {
    let forgotten = world.take_forgotten();
    if forgotten.is_empty() {
        return;
    }
    for (j, other) in clients.iter_mut().enumerate() {
        let inside = other
            .visit
            .as_ref()
            .is_some_and(|visit| forgotten.includes(visit.id()));
        if j != i && !other.gone && inside && !sites::put_out(other, world) {
            other.disconnect(world);
        }
    }
}

/// Welcomes client `i`, which has said hello. Anyone else playing its
/// character is disconnected first (and saved, so the character loads as
/// they left it): the newest connection wins, for coming back after a
/// connection drops before the server has noticed. `false` if the client
/// has gone.
fn join(clients: &mut [Client], i: usize, world: &mut ServerWorld, dev_tools: bool) -> bool {
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
    welcome(&mut clients[i], world, dev_tools)
}

/// Saves every character and site that has changed.
pub(crate) fn save_changes(mut connections: ResMut<Connections>, mut world: ResMut<ServerWorld>) {
    for client in &mut connections.0 {
        client.save(&mut world);
    }
    world.save_sites();
}

/// Every tick: tells everyone playing if the clock was moved, or has reached a
/// new whole hour (the tide follows it). A client that has gone is noticed when
/// its messages are next read.
pub(crate) fn tell_the_time(connections: Res<Connections>, mut world: ResMut<ServerWorld>) {
    let minutes = world.clock.minutes();
    let hour = (minutes / MINUTES_PER_HOUR) as u32;
    let moved = std::mem::take(&mut world.clock_moved);
    let new_hour = hour != world.hour;
    world.hour = hour;
    for client in connections.0.iter().filter(|client| client.welcomed) {
        if moved {
            client.link.send(&ServerMessage::Clock(minutes));
        }
        if new_hour {
            client.link.send(&ServerMessage::Hour(hour));
        }
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
        ClientMessage::Ping { sent } => client.link.send(&ServerMessage::Pong {
            sent,
            clock: world.clock.minutes(),
        }),
        // Meant for a map the player has since left.
        ClientMessage::Step { map, .. } if map != client.map => true,
        ClientMessage::Step { number, step, .. } => {
            client.last_step = Some(number);
            match client.visit {
                Some(_) => sites::stepped(client, step),
                None => world_map::stepped(client, world, step),
            }
        }
        ClientMessage::EnterSite { chunk } => sites::enter_poi(client, world, chunk),
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
        ClientMessage::DevToShowcase => world_map::to_showcase(client, world),
        ClientMessage::DevEnterSite {
            kind,
            seed,
            level,
            ruin,
        } => sites::dev_enter(client, world, kind, seed, level, ruin),
        ClientMessage::DevGenerateSite { kind, seed, ruin } => {
            sites::generate(client, world, kind, seed, ruin)
        }
        ClientMessage::DevSetCell { x, y, cell } => sites::set_cell(client, world, x, y, cell),
        ClientMessage::DevRevealPois => world_map::reveal_pois(client, world),
        ClientMessage::DevRevealMap => sites::reveal_level(client, world),
        ClientMessage::DevForgetMap => match client.visit {
            Some(_) => sites::forget_level(client, world),
            None => world_map::forget(client, world),
        },
        ClientMessage::DevForgetMapOnServer => match client.visit {
            Some(_) => sites::forget_site(client, world),
            None => world_map::forget_on_server(client, world),
        },
        ClientMessage::DevSnowLine(snow_line) => {
            world.snow_line = snow_line;
            client.link.send(&ServerMessage::SnowLine(snow_line))
        }
        ClientMessage::DevSetTime { minutes } => {
            world.clock.set(minutes);
            world.clock_moved = true;
            true
        }
    }
}

/// Loads the character (or starts a new one), and tells the client the
/// world and where they are; back into the site level they were on, if any.
fn welcome(client: &mut Client, world: &mut ServerWorld, dev_tools: bool) -> bool {
    client.joining = false;
    client.welcomed = true;
    let name = client.name.as_ref().expect("named on hello");
    client.character = world.storage.load_character(name).unwrap_or_default();
    let (x, y) = client.world_position();
    let welcome = ServerMessage::Welcome {
        world: world.info(),
        clock: world.clock.minutes(),
        snow_line: world.snow_line,
        x,
        y,
        dev_tools,
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
                    welcome(&mut self.client, &mut self.world, options.dev_tools);
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
        let mut app = World::new();
        app.insert_resource(connections);
        app.insert_resource(world);
        app.insert_resource(ServerOptions {
            dev_tools: true,
            exit_when_alone: false,
        });
        bevy::ecs::system::RunSystemOnce::run_system_once(&mut app, handle_messages).unwrap();
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
    fn forgetting_a_site_on_the_server_puts_only_others_out_at_the_poi() {
        let mut connections = Connections::default();
        let (first, link) = local_pair();
        connections.add(link, CharacterName::new("first"));
        let (second, link) = local_pair();
        connections.add(link, CharacterName::new("second"));
        first.send(&HELLO);
        second.send(&HELLO);
        let world = ServerWorld::new(WorldSeed::new(1));
        let (connections, world) = tick(connections, world);
        let enter = ClientMessage::DevEnterSite {
            kind: shared::poi::PoiKind::Castle,
            seed: 5,
            level: 0,
            ruin: 0,
        };
        first.send(&enter);
        second.send(&enter);
        let (connections, world) = tick(connections, world);
        answers(&first);
        answers(&second);

        first.send(&ClientMessage::DevForgetMapOnServer);
        let (connections, _world) = tick(connections, world);
        // The one who asked is still in it, arriving afresh.
        assert!(connections.0[0].visit.is_some());
        assert!(matches!(answers(&first)[0], ServerMessage::EnterLevel(_)));
        assert!(connections.0[1].visit.is_none());
        assert!(matches!(
            answers(&second)[0],
            ServerMessage::OnWorldMap { x: 0, y: 0 }
        ));
    }

    #[test]
    fn the_hour_and_moved_clocks_are_told_and_a_restart_carries_on() {
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

        let tell = |connections: Connections, world: ServerWorld| {
            let mut app = World::new();
            app.insert_resource(connections);
            app.insert_resource(world);
            bevy::ecs::system::RunSystemOnce::run_system_once(&mut app, tell_the_time).unwrap();
            (
                app.remove_resource::<Connections>().unwrap(),
                app.remove_resource::<ServerWorld>().unwrap(),
            )
        };
        // Within an hour says nothing; the next whole hour is told.
        world.hour = 40;
        world.clock.set(40.25 * MINUTES_PER_HOUR);
        let (connections, world) = tell(connections, world);
        assert_eq!(answers(&end), []);
        world.clock.set(41.0 * MINUTES_PER_HOUR);
        let (connections, mut world) = tell(connections, world);
        assert_eq!(answers(&end), [ServerMessage::Hour(41)]);
        // Moved by a dev tool, back as well: told the new time and hour.
        let keep = connections;
        let mut client = keep.0.into_iter().next().unwrap();
        let options = ServerOptions {
            dev_tools: true,
            exit_when_alone: false,
        };
        assert!(answer(
            &mut client,
            ClientMessage::DevSetTime { minutes: 1200.0 },
            &mut world,
            &options
        ));
        let connections = Connections(vec![client]);
        let (_connections, world) = tell(connections, world);
        let told = answers(&end);
        assert!(
            matches!(told[0], ServerMessage::Clock(minutes) if (1200.0..1210.0).contains(&minutes))
        );
        assert_eq!(told[1], ServerMessage::Hour(20));
        // Saved on stopping, and a restart carries on from there.
        drop(world);
        let restarted = open();
        assert_eq!(restarted.hour, 20);
        assert!((1200.0..1210.0).contains(&restarted.clock.minutes()));
    }

    #[test]
    fn saves_before_format_2_counted_hours() {
        let folder = crate::storage::tests::TestFolder::new();
        let seed = WorldSeed::new(1);
        let storage = crate::storage::Storage::in_folder(&folder.0, seed.id());
        storage.save_world(&crate::storage::WorldSave {
            format: 1,
            seed,
            world_map_version: shared::world::WORLD_MAP_VERSION,
            hour: 0,
            clock: 2.5,
        });
        let world = ServerWorld::open(seed, storage).unwrap();
        assert!((150.0..160.0).contains(&world.clock.minutes()));
        assert_eq!(world.hour, 2);
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
