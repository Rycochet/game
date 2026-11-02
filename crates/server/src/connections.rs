//! Connected clients: reading their messages, answering them, and saving
//! their characters.

use std::collections::{BTreeMap, BTreeSet, VecDeque};

use bevy::prelude::*;
use shared::creature::CreatureKind;
use shared::energy::{CHANGE_MAP_COST, Energy, NORMAL_SPEED, dressing_cost, site_step_cost};
use shared::id::Id;
use shared::item::{Inventory, Item, ItemKind, Made};
use shared::movement::{Step, world_step_cost};
use shared::poi::ChunkPos;
use shared::protocol::{
    CharacterId, ClientMessage, LoginMethod, PROTOCOL_VERSION, ServerLink, ServerMessage,
};
use shared::sheet::{Appearance, Sheet};
use shared::world::{MINUTES_PER_HOUR, WorldId};

use crate::accounts::{AccountId, Accounts, LoginKey, MAX_CHARACTERS};
use crate::sites::{self, Visit};
use crate::storage::CharacterWorldSave;
use crate::{ServerOptions, ServerWorld, world_map};

/// The clients connected to this server, and the accounts they log in to.
#[derive(Resource, Default)]
pub struct Connections {
    clients: Vec<Client>,
    accounts: Accounts,
}

impl Connections {
    pub(crate) fn new(accounts: Accounts) -> Self {
        Self {
            clients: Vec::new(),
            accounts,
        }
    }

    /// A newly connected client, which must log in.
    pub fn add(&mut self, link: ServerLink) {
        self.clients.push(Client::new(link, None));
    }

    /// A newly connected client that needs no login, playing this character:
    /// the embedded server's one player.
    pub fn add_playing(&mut self, link: ServerLink, character: CharacterId) {
        self.clients.push(Client::new(link, Some(character)));
    }

    pub fn is_empty(&self) -> bool {
        self.clients.is_empty()
    }

    /// Disconnects everyone, saving them (the server is stopping).
    pub(crate) fn disconnect_all(&mut self, world: &mut ServerWorld) {
        for client in &mut self.clients {
            client.disconnect(world);
        }
        self.clients.clear();
    }
}

/// What a client is told when saves can't be reached. Nothing has been lost.
const SAVES_TROUBLE: &str = "the server can't reach its saves just now: try again soon";

/// The most things a client can be waiting to do: a second or so of the quickest
/// steps. A client sending more is going faster than it can, and they are dropped,
/// and put right by `Position`.
const MOST_WAITING: usize = 64;

/// Something that takes time, waiting its turn. Things that depend on where the
/// player is have to wait behind the steps that lead there.
#[derive(Clone, Copy, Debug)]
enum Action {
    Step { number: u32, step: Step },
    EnterSite { chunk: ChunkPos },
    UseExit,
}

/// How far a client has got, from connecting to playing.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Phase {
    /// Nothing heard yet: `Hello` comes first.
    Greeting,
    /// Told how to log in; `Login` comes next.
    LoggingIn,
    /// Logged in and told its characters; choosing or making one.
    Choosing,
    /// Its character is decided, and `handle_messages` will welcome it.
    Joining,
    /// Welcomed, with its character loaded.
    Playing,
}

/// One connected client: its character, where it is and what it has been
/// sent.
pub struct Client {
    pub(crate) link: ServerLink,
    pub(crate) phase: Phase,
    /// The account it logged in to (none on an embedded server).
    account: Option<AccountId>,
    /// Its character: given when it connected, or decided by logging in.
    character_id: Option<CharacterId>,
    /// Whether it is to be disconnected (and was saved).
    gone: bool,
    /// What the character knows of this world and where they are in it: the
    /// pois and cells they know (`visit` is filled in on saving).
    pub(crate) character: CharacterWorldSave,
    /// What the character knows of other worlds, kept for saving.
    other_worlds: BTreeMap<WorldId, CharacterWorldSave>,
    /// What the character carries.
    pub(crate) inventory: Inventory,
    /// What others know this character by while they are playing.
    pub(crate) entity_id: Id,
    /// Who the client has been told is in sight, and where.
    pub(crate) seen_entities: BTreeMap<Id, (i64, i64)>,
    /// What the character is like.
    pub(crate) sheet: Sheet,
    /// How much it can do now, and how fast that comes back.
    energy: Energy,
    speed: u32,
    /// What it has asked to do and hasn't had the energy for yet, in order.
    waiting: VecDeque<Action>,
    /// What a character being made here asked to be, for when they first join.
    starting_as: Option<(CreatureKind, Appearance)>,
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
    pub(crate) fn new(link: ServerLink, character_id: Option<CharacterId>) -> Self {
        Self {
            link,
            phase: Phase::Greeting,
            account: None,
            character_id,
            gone: false,
            character: CharacterWorldSave::default(),
            other_worlds: BTreeMap::new(),
            inventory: Inventory::default(),
            entity_id: crate::ids::new_id(),
            seen_entities: BTreeMap::new(),
            sheet: Sheet::default(),
            energy: Energy::default(),
            speed: NORMAL_SPEED,
            waiting: VecDeque::new(),
            starting_as: None,
            unsaved: false,
            visit: None,
            world_chunks: BTreeSet::new(),
            reveal_pois: world_map::ALL_POIS_KNOWN,
            map: 0,
            last_step: None,
        }
    }

    /// Whether it is being disconnected.
    pub(crate) fn is_gone(&self) -> bool {
        self.gone
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
        self.waiting.clear();
        // The client forgets them too.
        self.seen_entities.clear();
    }

    /// What it carries has changed, which can change how fast it goes.
    fn update_speed(&mut self, tick: u64) {
        self.energy.settle(tick, self.speed);
        self.speed = self.sheet.speed(&self.inventory);
    }

    /// It has put this on or taken it off, which takes longer the heavier it is.
    fn spend_dressing(&mut self, item: shared::id::Id, tick: u64) {
        let grams = self.inventory.find(item).map_or(0, Item::total_weight);
        self.energy.spend(tick, self.speed, dressing_cost(grams));
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
        let Some(id) = &self.character_id else {
            return;
        };
        if self.phase != Phase::Playing || !self.unsaved {
            return;
        }
        self.character.visit = self.visit.as_ref().map(Visit::save);
        world.storage.save_character(
            id,
            world.seed.id(),
            &self.character,
            &self.other_worlds,
            &self.inventory,
            &self.sheet,
        );
        self.unsaved = false;
    }

    /// Saves can't be reached, so nothing about this client can be trusted to
    /// load: logs why, tells it, and answers `false` so that it is disconnected.
    pub(crate) fn refuse_for_saves(&self, error: &str) -> bool {
        error!("Refusing a client: {error}");
        self.link
            .send(&ServerMessage::Refused(SAVES_TROUBLE.into()));
        false
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
    let Connections { clients, accounts } = &mut *connections;
    for i in 0..clients.len() {
        let client = &mut clients[i];
        if client.gone {
            continue;
        }
        let stay = loop {
            match client.link.receive() {
                Ok(Some(message)) => {
                    if !answer(client, message, &mut world, accounts, &options) {
                        break false;
                    }
                    // The rest once it has been welcomed (below).
                    if client.phase == Phase::Joining {
                        break true;
                    }
                }
                Ok(None) => break act(client, &mut world) && client.send_position(),
                Err(error) => {
                    info!("A client left ({error:?})");
                    break false;
                }
            }
        };
        if !stay {
            client.disconnect(&mut world);
        } else if client.phase == Phase::Joining
            && !join(clients, i, &mut world, accounts, &options)
        {
            clients[i].disconnect(&mut world);
        }
        put_out_of_forgotten_sites(clients, i, &mut world);
    }
    clients.retain(|client| !client.gone);
}

/// Does what a client has asked, as far as it has the energy, in order. `false` if
/// the client has gone.
fn act(client: &mut Client, world: &mut ServerWorld) -> bool {
    let tick = world.action_clock.ticks();
    while client.energy.ready(tick, client.speed) {
        let Some(action) = client.waiting.pop_front() else {
            break;
        };
        let map = client.map;
        let (stayed, cost) = match action {
            Action::Step { number, step } => {
                client.last_step = Some(number);
                step_taken(client, world, step)
            }
            Action::EnterSite { chunk } => (sites::enter_poi(client, world, chunk), 0),
            Action::UseExit => (sites::use_exit(client, world), 0),
        };
        if !stayed {
            return false;
        }
        let cost = if client.map == map {
            cost
        } else {
            CHANGE_MAP_COST
        };
        client.energy.spend(tick, client.speed, cost);
    }
    true
}

/// Takes a step, if it is allowed: whether the client is still there, and what it
/// cost, which is nothing if it was refused.
fn step_taken(client: &mut Client, world: &mut ServerWorld, step: Step) -> (bool, u32) {
    match &client.visit {
        Some(visit) => {
            let from = visit.position();
            let stayed = sites::stepped(client, step);
            let to = client.visit.as_ref().map_or(from, Visit::position);
            (
                stayed,
                if to == from {
                    0
                } else {
                    site_step_cost(from, to)
                },
            )
        }
        None => {
            let from = client.world_position();
            let stayed = world_map::stepped(client, world, step);
            if client.world_position() == from {
                return (stayed, 0);
            }
            // Where it led before any wrapping round the planet.
            let to = step.from(from);
            let conditions = world.conditions();
            let ends = (
                world.ground.elevation(from.0, from.1, conditions),
                world.ground.elevation(to.0, to.1, conditions),
            );
            (stayed, world_step_cost(from, to, ends))
        }
    }
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

/// Welcomes client `i`, whose character is decided. Anyone else playing that
/// account or character is disconnected first (and saved, so the character
/// loads as they left it): the newest connection wins, for coming back after a
/// connection drops before the server has noticed. One character per account
/// is online at a time. `false` if the client has gone.
fn join(
    clients: &mut [Client],
    i: usize,
    world: &mut ServerWorld,
    accounts: &mut Accounts,
    options: &ServerOptions,
) -> bool {
    let account = clients[i].account.clone();
    let character = clients[i].character_id.clone();
    for (j, other) in clients.iter_mut().enumerate() {
        let same_account = account.is_some() && other.account == account;
        let same = same_account || other.character_id == character;
        if j != i && !other.gone && other.phase == Phase::Playing && same {
            info!("{character:?} connected again: dropping the older connection");
            other.link.send(&ServerMessage::Refused(
                "this account is playing somewhere else now".into(),
            ));
            other.disconnect(world);
        }
    }
    if let (Some(account), Some(character)) = (&account, &character)
        && let Err(error) = accounts.played(account, character)
    {
        return clients[i].refuse_for_saves(&error);
    }
    welcome(&mut clients[i], world, options)
}

/// Saves every character and site that has changed.
pub(crate) fn save_changes(mut connections: ResMut<Connections>, mut world: ResMut<ServerWorld>) {
    for client in &mut connections.clients {
        client.save(&mut world);
    }
    world.save_sites();
}

/// Every tick: mobs do what they are doing.
pub(crate) fn run_mobs(mut connections: ResMut<Connections>, mut world: ResMut<ServerWorld>) {
    crate::entities::run_mobs(&mut connections.clients, &mut world);
}

/// Every tick: tells everyone who has come into or gone out of their sight, or
/// moved. Who is on a level stays while it is in use.
pub(crate) fn sync_entities(mut connections: ResMut<Connections>, mut world: ResMut<ServerWorld>) {
    crate::entities::sync(&mut connections.clients, &mut world);
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
    let playing = |client: &&Client| client.phase == Phase::Playing;
    for client in connections.clients.iter().filter(playing) {
        if moved {
            client.link.send(&ServerMessage::Clock(minutes));
        }
        if new_hour {
            client.link.send(&ServerMessage::Hour(hour));
        }
    }
}

/// A client has logged in: a guest goes straight to their one character, anyone
/// else is told theirs to choose from.
fn logged_in(client: &mut Client, key: &LoginKey, accounts: &mut Accounts) -> bool {
    let answered = (|| {
        let id = accounts.log_in(key)?;
        let account = accounts.account(&id)?;
        let answered = if account.guest {
            client.character_id = Some(accounts.guest_character(&id)?);
            client.phase = Phase::Joining;
            true
        } else {
            client.phase = Phase::Choosing;
            client.link.send(&ServerMessage::Characters {
                list: account.characters.clone(),
                can_create: account.characters.len() < MAX_CHARACTERS,
            })
        };
        client.account = Some(id);
        Ok::<bool, String>(answered)
    })();
    answered.unwrap_or_else(|error| client.refuse_for_saves(&error))
}

/// Answers one message; `false` if the client should be disconnected.
pub(crate) fn answer(
    client: &mut Client,
    message: ClientMessage,
    world: &mut ServerWorld,
    accounts: &mut Accounts,
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
        ClientMessage::Hello { .. } if client.phase != Phase::Greeting => {
            refuse(client, "hello again?".into())
        }
        // Needs no login: welcomed by `handle_messages`, which can see other
        // clients.
        ClientMessage::Hello { .. } if client.character_id.is_some() => {
            client.phase = Phase::Joining;
            true
        }
        ClientMessage::Hello { .. } => {
            client.phase = Phase::LoggingIn;
            let methods = if options.guests {
                vec![LoginMethod::Guest]
            } else {
                Vec::new()
            };
            client.link.send(&ServerMessage::LoginOptions { methods })
        }
        ClientMessage::Login(_) if client.phase != Phase::LoggingIn => {
            refuse(client, "not the time to log in".into())
        }
        ClientMessage::Login(credentials) => {
            let key = LoginKey::from(&credentials);
            if matches!(key, LoginKey::Guest(_)) && !options.guests {
                return refuse(client, "this server doesn't take guests".into());
            }
            logged_in(client, &key, accounts)
        }
        ClientMessage::ChooseCharacter { id } => {
            let Some(account) = client
                .account
                .clone()
                .filter(|_| client.phase == Phase::Choosing)
            else {
                return refuse(client, "not the time to choose a character".into());
            };
            // A client can ask for anything: only its own will do.
            let characters = match accounts.account(&account) {
                Ok(account) => &account.characters,
                Err(error) => return client.refuse_for_saves(&error),
            };
            if !characters.iter().any(|character| character.id == id) {
                return refuse(client, "that isn't one of this account's characters".into());
            }
            client.character_id = Some(id);
            client.phase = Phase::Joining;
            true
        }
        ClientMessage::CreateCharacter {
            name,
            kind,
            appearance,
        } => {
            let Some(account) = client
                .account
                .clone()
                .filter(|_| client.phase == Phase::Choosing)
            else {
                return refuse(client, "not the time to make a character".into());
            };
            // A client can ask for anything: the server decides what may be played.
            if !kind.def().playable {
                return refuse(client, format!("{} can't be played", kind.def().name));
            }
            if !appearance.suits(kind) {
                return refuse(
                    client,
                    "that look doesn't suit that kind of creature".into(),
                );
            }
            match accounts.create_character(&account, &name) {
                Ok(id) => {
                    client.character_id = Some(id);
                    client.starting_as = Some((kind, appearance));
                    client.phase = Phase::Joining;
                    true
                }
                Err(reason) => refuse(client, reason),
            }
        }
        _ if client.phase != Phase::Playing => refuse(client, "say hello first".into()),
        ClientMessage::Ping { sent } => client.link.send(&ServerMessage::Pong {
            sent,
            clock: world.clock.minutes(),
        }),
        // Meant for a map the player has since left.
        ClientMessage::Step { map, .. } if map != client.map => true,
        ClientMessage::Step { number, step, .. } => {
            if client.waiting.len() < MOST_WAITING {
                client.waiting.push_back(Action::Step { number, step });
            } else {
                // Its `Position` puts the client right.
                client.last_step = Some(number);
            }
            true
        }
        ClientMessage::EnterSite { chunk } => {
            client.waiting.push_back(Action::EnterSite { chunk });
            true
        }
        ClientMessage::UseExit => {
            client.waiting.push_back(Action::UseExit);
            true
        }
        ClientMessage::SetPaused(_) if !options.pausable => {
            warn!("A client asked to pause, which this server doesn't allow");
            true
        }
        ClientMessage::SetPaused(paused) => {
            if paused {
                world.clock.pause();
            } else {
                world.clock.resume();
            }
            client.link.send(&ServerMessage::Paused {
                paused,
                clock: world.clock.minutes(),
            })
        }
        ClientMessage::MoveItem {
            item,
            to,
            x,
            y,
            rotation,
        } => {
            let was_worn = client.inventory.is_worn(item);
            let tick = world.action_clock.ticks();
            match client.inventory.move_item(item, to, x, y, rotation) {
                Ok(()) => {
                    client.unsaved = true;
                    if was_worn {
                        client.spend_dressing(item, tick);
                    }
                }
                Err(why) => warn!("Refused moving item {item:?} to {to:?}: {why:?}"),
            }
            client.update_speed(world.action_clock.ticks());
            client
                .link
                .send(&ServerMessage::Inventory(client.inventory.clone()))
        }
        ClientMessage::WearItem { item } => {
            let tick = world.action_clock.ticks();
            match client.inventory.wear(item) {
                Ok(_) => {
                    client.unsaved = true;
                    client.spend_dressing(item, tick);
                }
                Err(why) => warn!("Refused wearing {item:?}: {why:?}"),
            }
            client.update_speed(world.action_clock.ticks());
            client
                .link
                .send(&ServerMessage::Inventory(client.inventory.clone()))
        }
        ClientMessage::DestroyItem { item } => {
            if client.inventory.destroy(item) {
                client.unsaved = true;
            } else {
                warn!("Refused destroying {item:?}, which isn't carried");
            }
            client.update_speed(world.action_clock.ticks());
            client
                .link
                .send(&ServerMessage::Inventory(client.inventory.clone()))
        }
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
        ClientMessage::DevGiveItem {
            kind,
            size,
            magical,
        } => {
            let new = made_for(&client.inventory, kind, size, magical);
            match client.inventory.give(new) {
                Ok(()) => client.unsaved = true,
                Err(item) => info!("No room for a new {:?}", item.kind),
            }
            client.update_speed(world.action_clock.ticks());
            client
                .link
                .send(&ServerMessage::Inventory(client.inventory.clone()))
        }
        ClientMessage::DevSetCreature(kind) => {
            for item in client.inventory.change_body(kind) {
                info!("No room for {:?} on becoming {kind:?}: lost", item.kind);
            }
            client.sheet = Sheet::new(kind);
            client.unsaved = true;
            client
                .link
                .send(&ServerMessage::Sheet(client.sheet.clone()))
                && {
                    client.update_speed(world.action_clock.ticks());
                    client
                        .link
                        .send(&ServerMessage::Inventory(client.inventory.clone()))
                }
        }
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

/// A new item of this kind, made for this size or the wearer's, and their fingers.
fn made_for(
    inventory: &Inventory,
    kind: ItemKind,
    size: Option<shared::creature::Size>,
    magical: bool,
) -> Item {
    let body = inventory.creature.def();
    Item {
        made: Made::for_body(kind, size.unwrap_or(body.size), body.fingers(), magical),
        ..Item::new(crate::ids::new_id(), kind)
    }
}

/// What a joining character is like: as saved; or as they asked when being made;
/// or for a character from before sheets, plain for their kind.
fn start_sheet(
    saved: Option<Sheet>,
    asked: Option<(CreatureKind, Appearance)>,
    inventory: &mut Inventory,
) -> Sheet {
    if let Some(sheet) = saved {
        return sheet;
    }
    match asked {
        Some((kind, appearance)) => {
            inventory.change_body(kind);
            Sheet {
                appearance,
                ..Sheet::new(kind)
            }
        }
        None => Sheet::new(inventory.creature),
    }
}

/// A new character's clothes. Characters from before clothes get them too, unless
/// they are already wearing something.
fn give_first_clothes(inventory: &mut Inventory) {
    if std::mem::replace(&mut inventory.outfitted, true) || !inventory.worn.is_empty() {
        return;
    }
    for kind in [ItemKind::TShirt, ItemKind::Trousers, ItemKind::Shoes] {
        // Straight on, as the pockets may be full. Only for bodies they fit: other
        // kinds of creature start with none.
        if inventory.creature == shared::creature::CreatureKind::Human {
            let item = made_for(inventory, kind, None, false);
            inventory.worn.push(item);
        }
    }
}

/// Loads the character (or starts a new one), and tells the client the
/// world and where they are; back into the site level they were on, if any.
fn welcome(client: &mut Client, world: &mut ServerWorld, options: &ServerOptions) -> bool {
    let id = client
        .character_id
        .as_ref()
        .expect("decided before joining");
    let loaded = world
        .storage
        .load_character(id, world.seed.id())
        .and_then(|character| {
            let inventory = world.storage.load_inventory(id)?;
            Ok((character, inventory, world.storage.load_sheet(id)?))
        });
    // Not `Playing` yet, so nothing blank can be saved over the real character.
    match loaded {
        Ok((character, inventory, sheet)) => {
            (client.character, client.other_worlds) = character;
            client.inventory = inventory;
            client.sheet = start_sheet(sheet, client.starting_as.take(), &mut client.inventory);
            give_first_clothes(&mut client.inventory);
            client.speed = client.sheet.speed(&client.inventory);
            client.unsaved = true;
        }
        Err(error) => return client.refuse_for_saves(&error),
    }
    client.phase = Phase::Playing;
    let (x, y) = client.world_position();
    let welcome = ServerMessage::Welcome {
        world: world.info(),
        clock: world.clock.minutes(),
        snow_line: world.snow_line,
        x,
        y,
        dev_tools: options.dev_tools,
        can_pause: options.pausable,
    };
    if !client.link.send(&welcome)
        || !client
            .link
            .send(&ServerMessage::Sheet(client.sheet.clone()))
        || !client
            .link
            .send(&ServerMessage::Inventory(client.inventory.clone()))
    {
        return false;
    }
    match client.character.visit.take() {
        Some(visit) => sites::resume(client, world, visit),
        None => world_map::look_around(client, world),
    }
}

#[cfg(test)]
pub(crate) mod tests {
    use std::sync::Arc;
    use std::sync::atomic::Ordering;
    use std::time::Duration;

    use shared::item::{Holder, Rotation};
    use shared::movement::{Step, steps_between};
    use shared::poi::PoiKind;
    use shared::protocol::{ClientLink, Credentials, GuestToken, local_pair};
    use shared::world::WorldSeed;

    use super::*;
    use crate::save_queue::{SaveQueue, tests::MemoryBackend};
    use crate::saves::SaveKey;
    use crate::storage::{SiteId, SiteSave, Storage, tests::TestFolder};

    /// A server world, a client connected to it, and the client's end.
    pub(crate) struct TestClient {
        pub world: ServerWorld,
        accounts: Accounts,
        pub client: Client,
        pub end: ClientLink,
        /// The answers to `Hello`.
        pub welcome: Vec<ServerMessage>,
        /// Whether the server allows dev tools (it does, unless set).
        pub dev_tools: bool,
        /// Whether the server pauses when asked (it doesn't, unless set).
        pub pausable: bool,
        /// Whether as much time passes as what is asked takes (it does, unless set).
        pub time_passes: bool,
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
                accounts: Accounts::default(),
                client: Client::new(server_link, CharacterId::new("tester")),
                end,
                welcome: Vec::new(),
                dev_tools: true,
                pausable: false,
                time_passes: true,
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
                pausable: self.pausable,
                guests: true,
            };
            for message in messages {
                answer(
                    &mut self.client,
                    message.clone(),
                    &mut self.world,
                    &mut self.accounts,
                    &options,
                );
                if self.client.phase == Phase::Joining {
                    welcome(&mut self.client, &mut self.world, &options);
                }
                // A long walk in one go shouldn't fill what can wait.
                self.let_time_pass();
            }
            self.let_time_pass();
            self.client.send_position();
            crate::entities::sync(std::slice::from_mut(&mut self.client), &mut self.world);
            std::iter::from_fn(|| self.end.receive().unwrap()).collect()
        }

        /// As much time as what was asked takes, unless a test is watching time.
        fn let_time_pass(&mut self) {
            act(&mut self.client, &mut self.world);
            while self.time_passes && !self.client.waiting.is_empty() {
                self.world.action_clock.advance(1);
                act(&mut self.client, &mut self.world);
            }
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

    const GUEST: ClientMessage = ClientMessage::Login(Credentials::Guest(GuestToken([1; 32])));

    fn give(kind: ItemKind) -> ClientMessage {
        ClientMessage::DevGiveItem {
            kind,
            size: None,
            magical: false,
        }
    }

    /// What the last `Inventory` among these answers says.
    fn inventory(answers: &[ServerMessage]) -> Inventory {
        answers
            .iter()
            .rev()
            .find_map(|answer| match answer {
                ServerMessage::Inventory(inventory) => Some(inventory.clone()),
                _ => None,
            })
            .expect("an inventory among the answers")
    }

    #[test]
    fn a_character_is_told_what_they_carry_on_joining() {
        let first = inventory(&TestClient::new().welcome);
        assert!(first.pockets.is_empty());
        let kinds: Vec<_> = first.worn.iter().map(|item| item.kind).collect();
        assert_eq!(
            kinds,
            [ItemKind::TShirt, ItemKind::Trousers, ItemKind::Shoes]
        );
    }

    #[test]
    fn items_move_by_the_rules_and_a_refused_move_is_put_right() {
        let mut test = TestClient::new();
        let answers = test.send(&[give(ItemKind::Pouch)]);
        let pouch = inventory(&answers).pockets[0].item.id;
        let answers = test.send(&[give(ItemKind::Stone)]);
        let carried = inventory(&answers);
        let stone = carried.pockets[1].item.id;

        // Too big for the pouch: the answer still says where it really is.
        let refused = ClientMessage::MoveItem {
            item: stone,
            to: Holder::Inside(pouch),
            x: 0,
            y: 0,
            rotation: Rotation::None,
        };
        assert_eq!(inventory(&test.send(&[refused])), carried);

        let answers = test.send(&[ClientMessage::MoveItem {
            item: stone,
            to: Holder::Pockets,
            x: 4,
            y: 2,
            rotation: Rotation::None,
        }]);
        let carried = inventory(&answers);
        let (_, placed) = carried.placed(stone).expect("still carried");
        assert_eq!((placed.x, placed.y), (4, 2));
        assert!(test.client.unsaved);
    }

    #[test]
    fn destroying_takes_what_is_carried_and_only_that() {
        let mut test = TestClient::new();
        let answers = test.send(&[give(ItemKind::Coin)]);
        let coin = inventory(&answers).pockets[0].item.id;
        let destroy = ClientMessage::DestroyItem { item: coin };
        assert!(
            inventory(&test.send(std::slice::from_ref(&destroy)))
                .pockets
                .is_empty()
        );
        assert!(test.client.unsaved);
        // Gone already, so nothing changes, and the client is still told.
        test.client.unsaved = false;
        assert!(inventory(&test.send(&[destroy])).pockets.is_empty());
        assert!(!test.client.unsaved);
    }

    #[test]
    fn clothes_go_on_by_the_rules_and_come_off_with_a_move() {
        let mut test = TestClient::new();
        let answers = test.send(&[give(ItemKind::Boots)]);
        let boots = inventory(&answers).pockets[0].item.id;
        let wear = ClientMessage::WearItem { item: boots };
        let worn = inventory(&test.send(std::slice::from_ref(&wear)));
        assert!(worn.is_worn(boots));
        assert!(worn.pockets.is_empty());

        // The shoes are under the boots, so a refused move is put right.
        let shoes = worn
            .worn
            .iter()
            .find(|item| item.kind == ItemKind::Shoes)
            .unwrap()
            .id;
        let take_off = |item| ClientMessage::MoveItem {
            item,
            to: Holder::Pockets,
            x: 0,
            y: 0,
            rotation: Rotation::None,
        };
        assert!(inventory(&test.send(&[take_off(shoes)])).is_worn(shoes));
        let answers = test.send(&[take_off(boots)]);
        assert!(!inventory(&answers).is_worn(boots));
    }

    #[test]
    fn a_new_body_takes_everything_off_and_is_kept() {
        let folder = TestFolder::new();
        let seed = WorldSeed::new(1);
        let open = || ServerWorld::open(seed, Storage::in_folder(&folder.0, seed.id())).unwrap();
        let mut test = TestClient::of(open());
        let answers = test.send(&[ClientMessage::DevSetCreature(CreatureKind::Kobold)]);
        let changed = inventory(&answers);
        assert_eq!(changed.creature, CreatureKind::Kobold);
        assert!(changed.worn.is_empty());
        assert_eq!(changed.pockets_size, (4, 3));
        test.leave();
        // They aren't dressed again on coming back.
        assert_eq!(inventory(&TestClient::of(open()).welcome), changed);
    }

    #[test]
    fn items_are_made_for_the_wearer_or_the_size_asked_for() {
        let mut test = TestClient::new();
        test.send(&[ClientMessage::DevSetCreature(CreatureKind::Giant)]);
        let answers = test.send(&[give(ItemKind::Gloves)]);
        let gloves = inventory(&answers).pockets.last().unwrap().item.made;
        assert_eq!(gloves.size, shared::creature::Size::Giant);
        assert_eq!(gloves.fingers, Some(10));
        let large = ClientMessage::DevGiveItem {
            kind: ItemKind::Coin,
            size: Some(shared::creature::Size::Large),
            magical: true,
        };
        let made = inventory(&test.send(&[large]))
            .pockets
            .last()
            .unwrap()
            .item
            .made;
        assert_eq!(made.size, shared::creature::Size::Large);
        assert!(made.magical);
    }

    #[test]
    fn a_character_is_told_their_sheet_and_a_new_body_gives_a_new_one() {
        let sheet = |answers: &[ServerMessage]| {
            answers.iter().rev().find_map(|answer| match answer {
                ServerMessage::Sheet(sheet) => Some(sheet.clone()),
                _ => None,
            })
        };
        let mut test = TestClient::new();
        assert_eq!(sheet(&test.welcome), Some(Sheet::new(CreatureKind::Human)));
        let answers = test.send(&[ClientMessage::DevSetCreature(CreatureKind::Giant)]);
        assert_eq!(sheet(&answers), Some(Sheet::new(CreatureKind::Giant)));
    }

    #[test]
    fn steps_wait_for_energy_in_order_and_time_brings_them() {
        let mut test = TestClient::new();
        enter_a_room(&mut test);
        test.time_passes = false;
        let before = test.client.visit.as_ref().unwrap().position();
        // Three straight steps: the first at once, the others waiting.
        let east = (1, 0);
        test.steps(&[east, east, east]);
        let after_first = (before.0 + 1, before.1);
        assert_eq!(test.client.visit.as_ref().unwrap().position(), after_first);
        assert_eq!(test.client.waiting.len(), 2);

        // A step costs 1⅔ ticks: one more after 2 ticks, and the last after 2 more.
        test.world.action_clock.advance(1);
        test.send(&[]);
        assert_eq!(test.client.waiting.len(), 2);
        test.world.action_clock.advance(1);
        test.send(&[]);
        assert_eq!(test.client.waiting.len(), 1);
        test.world.action_clock.advance(2);
        let answers = test.send(&[]);
        assert!(test.client.waiting.is_empty());
        assert_eq!(
            test.client.visit.as_ref().unwrap().position(),
            (before.0 + 3, before.1)
        );
        assert!(
            answers
                .iter()
                .any(|answer| matches!(answer, ServerMessage::Position { .. }))
        );
    }

    /// Into a made-up site's wide room, away from walls.
    fn enter_a_room(test: &mut TestClient) {
        test.send(&[ClientMessage::DevEnterSite {
            kind: shared::poi::PoiKind::Castle,
            seed: 1,
            level: 0,
            ruin: 0,
        }]);
        // Somewhere with room for three steps east.
        let (x, y) = test.client.visit.as_ref().unwrap().room_to_walk_east(3);
        test.send(&[ClientMessage::DevTeleport { x, y }]);
    }

    #[test]
    fn exits_wait_behind_the_steps_leading_to_them() {
        let mut test = TestClient::new();
        test.time_passes = false;
        // On the world map, where a chunk with a poi is further than one step.
        test.send(&[ClientMessage::DevTeleport { x: 0, y: 0 }]);
        test.client.energy.spend(0, 210, 1000);
        test.send(&[ClientMessage::UseExit]);
        assert_eq!(test.client.waiting.len(), 1);
        assert!(matches!(test.client.waiting[0], Action::UseExit));
    }

    #[test]
    fn too_many_steps_are_dropped_and_the_client_put_right() {
        let mut test = TestClient::new();
        test.time_passes = false;
        let steps = vec![(1, 0); MOST_WAITING + 10];
        let answers = test.steps(&steps);
        assert!(test.client.waiting.len() <= MOST_WAITING);
        assert!(
            answers
                .iter()
                .any(|answer| matches!(answer, ServerMessage::Position { .. }))
        );
    }

    #[test]
    fn heavy_loads_slow_a_character_and_dressing_takes_time() {
        let mut test = TestClient::new();
        let normal = test.client.speed;
        let stones = vec![give(ItemKind::Stone); 5];
        test.send(&stones);
        assert!(test.client.speed < normal);

        let mut test = TestClient::new();
        test.time_passes = false;
        let answers = test.send(&[give(ItemKind::Hat)]);
        let hat = inventory(&answers).pockets[0].item.id;
        test.send(&[ClientMessage::WearItem { item: hat }]);
        assert!(!test.client.energy.ready(0, test.client.speed));
    }

    #[test]
    fn giving_an_item_needs_dev_tools_and_room() {
        let mut test = TestClient::new();
        test.dev_tools = false;
        let give = give(ItemKind::Coin);
        assert!(test.send(std::slice::from_ref(&give)).is_empty());

        test.dev_tools = true;
        let answers = test.send(&vec![give; 30]);
        // Six by four pockets, and no more.
        assert_eq!(inventory(&answers).pockets.len(), 24);
    }

    #[test]
    fn what_is_carried_comes_back_with_its_ids_after_a_restart() {
        let folder = TestFolder::new();
        let seed = WorldSeed::new(1);
        let open = || ServerWorld::open(seed, Storage::in_folder(&folder.0, seed.id())).unwrap();
        let mut test = TestClient::of(open());
        let answers = test.send(&[give(ItemKind::Backpack)]);
        let carried = inventory(&answers);
        test.leave();
        assert_eq!(inventory(&TestClient::of(open()).welcome), carried);
    }

    /// One real tick's messages and sight, in place.
    pub(crate) fn tick_both(connections: &mut Connections, world: &mut ServerWorld) {
        let mut app = World::new();
        app.insert_resource(std::mem::take(connections));
        let seed = world.seed;
        app.insert_resource(std::mem::replace(world, ServerWorld::new(seed)));
        app.insert_resource(ServerOptions {
            dev_tools: true,
            exit_when_alone: false,
            pausable: false,
            guests: true,
        });
        bevy::ecs::system::RunSystemOnce::run_system_once(&mut app, handle_messages).unwrap();
        bevy::ecs::system::RunSystemOnce::run_system_once(&mut app, sync_entities).unwrap();
        *connections = app.remove_resource::<Connections>().unwrap();
        *world = app.remove_resource::<ServerWorld>().unwrap();
    }

    /// Runs `handle_messages` once over these connections.
    fn tick(connections: Connections, world: ServerWorld) -> (Connections, ServerWorld) {
        tick_with(connections, world, true)
    }

    /// `tick`, on a server that does or doesn't take guests.
    fn tick_with(
        connections: Connections,
        world: ServerWorld,
        guests: bool,
    ) -> (Connections, ServerWorld) {
        let mut app = World::new();
        app.insert_resource(connections);
        app.insert_resource(world);
        app.insert_resource(ServerOptions {
            dev_tools: true,
            exit_when_alone: false,
            pausable: false,
            guests,
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

    /// A client that has logged in to an account that isn't a guest's, and been
    /// told its characters, with the list.
    fn logged_in_to_account(
        connections: &mut Connections,
        key: &'static str,
    ) -> (ClientLink, Vec<ServerMessage>) {
        let (end, link) = local_pair();
        connections.add(link);
        let client = connections.clients.last_mut().unwrap();
        client.phase = Phase::LoggingIn;
        assert!(logged_in(
            client,
            &LoginKey::Test(key),
            &mut connections.accounts
        ));
        let told = answers(&end);
        (end, told)
    }

    #[test]
    fn logging_in_comes_after_hello_and_before_anything_else() {
        let world = ServerWorld::new(WorldSeed::new(1));
        let mut connections = Connections::default();
        let (too_soon, link) = local_pair();
        connections.add(link);
        too_soon.send(&GUEST);
        let (end, link) = local_pair();
        connections.add(link);
        end.send(&HELLO);
        end.send(&ClientMessage::UseExit);
        let (mut connections, world) = tick(connections, world);
        assert!(matches!(
            answers(&too_soon)[..],
            [ServerMessage::Refused(_)]
        ));
        let told = answers(&end);
        assert_eq!(
            told[0],
            ServerMessage::LoginOptions {
                methods: vec![LoginMethod::Guest]
            }
        );
        assert!(matches!(told[1], ServerMessage::Refused(_)));
        assert!(connections.clients.is_empty());

        // Logged in, a second login is refused.
        let (end, link) = local_pair();
        connections.add(link);
        end.send(&HELLO);
        end.send(&GUEST);
        let (connections, world) = tick(connections, world);
        assert!(matches!(answers(&end)[1], ServerMessage::Welcome { .. }));
        end.send(&GUEST);
        let (connections, _world) = tick(connections, world);
        assert!(matches!(answers(&end)[..], [ServerMessage::Refused(_)]));
        assert!(connections.clients.is_empty());
    }

    #[test]
    fn a_server_can_turn_guests_away() {
        let world = ServerWorld::new(WorldSeed::new(1));
        let mut connections = Connections::default();
        let (end, link) = local_pair();
        connections.add(link);
        end.send(&HELLO);
        end.send(&GUEST);
        let (connections, _world) = tick_with(connections, world, false);
        let told = answers(&end);
        assert_eq!(
            told[0],
            ServerMessage::LoginOptions {
                methods: Vec::new()
            }
        );
        assert!(matches!(told[1], ServerMessage::Refused(_)));
        assert!(connections.clients.is_empty());
    }

    #[test]
    fn a_new_character_is_what_they_asked_if_it_is_allowed() {
        use std::collections::BTreeMap;

        use shared::sheet::Trait;

        let make = |key, kind, appearance| {
            let world = ServerWorld::new(WorldSeed::new(1));
            let mut connections = Connections::default();
            let (end, _) = logged_in_to_account(&mut connections, key);
            end.send(&ClientMessage::CreateCharacter {
                name: "Some One".into(),
                kind,
                appearance,
            });
            let _ = tick(connections, world);
            answers(&end)
        };
        let red = Appearance(BTreeMap::from([(Trait::HairColour, 3)]));

        // Kinds nobody may play, and looks that don't suit them.
        let told = make("a", CreatureKind::Horse, Appearance::default());
        assert!(matches!(told[..], [ServerMessage::Refused(_)]), "{told:?}");
        let told = make("b", CreatureKind::Kobold, red.clone());
        assert!(matches!(told[..], [ServerMessage::Refused(_)]), "{told:?}");

        let told = make("c", CreatureKind::Giant, red.clone());
        let sheet = told.iter().find_map(|answer| match answer {
            ServerMessage::Sheet(sheet) => Some(sheet.clone()),
            _ => None,
        });
        assert_eq!(
            sheet,
            Some(Sheet {
                appearance: red,
                ..Sheet::new(CreatureKind::Giant)
            })
        );
        let carried = inventory(&told);
        assert_eq!(carried.creature, CreatureKind::Giant);
        assert_eq!(carried.pockets_size, CreatureKind::Giant.def().pockets);
    }

    #[test]
    fn accounts_choose_or_make_a_character_and_only_their_own() {
        let world = ServerWorld::new(WorldSeed::new(1));
        let mut connections = Connections::default();

        // New: nothing to choose, and room to make one.
        let (end, told) = logged_in_to_account(&mut connections, "someone");
        assert_eq!(
            told,
            [ServerMessage::Characters {
                list: Vec::new(),
                can_create: true
            }]
        );
        end.send(&ClientMessage::CreateCharacter {
            kind: CreatureKind::Human,
            appearance: Appearance::default(),
            name: "Some One".into(),
        });
        let (connections, world) = tick(connections, world);
        assert!(matches!(answers(&end)[0], ServerMessage::Welcome { .. }));
        let mut connections = connections;

        // Back later: there they are. (Dropping the first connection is `join`'s
        // job, when this one chooses.)
        let (second, told) = logged_in_to_account(&mut connections, "someone");
        let [ServerMessage::Characters { list, can_create }] = &told[..] else {
            panic!("no characters: {told:?}");
        };
        assert!(*can_create);
        assert_eq!(list[0].name, "Some One");
        assert!(list[0].last_played > 0);
        let theirs = list[0].id.clone();

        // Someone else's can't be chosen.
        let (other, _) = logged_in_to_account(&mut connections, "someone else");
        other.send(&ClientMessage::ChooseCharacter { id: theirs.clone() });
        // Their own can, and an account plays one at a time.
        second.send(&ClientMessage::ChooseCharacter { id: theirs });
        let (connections, _world) = tick(connections, world);
        assert!(matches!(answers(&other)[..], [ServerMessage::Refused(_)]));
        assert!(matches!(answers(&second)[0], ServerMessage::Welcome { .. }));
        assert!(matches!(answers(&end)[..], [ServerMessage::Refused(_)]));
        assert_eq!(connections.clients.len(), 1);
    }

    #[test]
    fn choosing_is_only_for_accounts_about_to_choose() {
        let world = ServerWorld::new(WorldSeed::new(1));
        let mut connections = Connections::default();
        let (end, link) = local_pair();
        connections.add(link);
        end.send(&HELLO);
        end.send(&GUEST);
        let (connections, world) = tick(connections, world);
        answers(&end);
        end.send(&ClientMessage::CreateCharacter {
            kind: CreatureKind::Human,
            appearance: Appearance::default(),
            name: "Another".into(),
        });
        let (connections, _world) = tick(connections, world);
        assert!(matches!(answers(&end)[..], [ServerMessage::Refused(_)]));
        assert!(connections.clients.is_empty());
    }

    /// A world saved in a backend that can be told to fail, and its queue.
    fn world_in_memory() -> (Arc<MemoryBackend>, SaveQueue, ServerWorld) {
        let backend = Arc::new(MemoryBackend::default());
        let queue = SaveQueue::new(backend.clone());
        let seed = WorldSeed::new(1);
        let storage = Storage::new(queue.clone(), seed.id());
        let world = ServerWorld::open(seed, storage).unwrap();
        (backend, queue, world)
    }

    fn refused(answers: &[ServerMessage]) -> bool {
        answers
            .iter()
            .any(|answer| matches!(answer, ServerMessage::Refused(_)))
    }

    #[test]
    fn a_character_that_cant_be_reached_is_refused_and_not_saved_over() {
        let (backend, queue, world) = world_in_memory();
        let mut test = TestClient::of(world);
        test.send(&[ClientMessage::DevTeleport { x: 5, y: 6 }]);
        let world = test.leave();
        assert!(queue.flush(Duration::from_secs(5)));
        let key = SaveKey::Character(CharacterId::new("tester").unwrap());
        let before = backend.holds(&key);
        assert!(before.is_some());

        backend.cant_read.store(true, Ordering::Relaxed);
        let test = TestClient::of(world);
        assert!(refused(&test.welcome));
        let world = test.leave();
        assert!(queue.flush(Duration::from_secs(5)));
        assert_eq!(backend.holds(&key), before);

        // Back to normal: there they are.
        backend.cant_read.store(false, Ordering::Relaxed);
        let test = TestClient::of(world);
        assert!(!refused(&test.welcome));
        assert_eq!(test.client.world_position(), (5, 6));
    }

    #[test]
    fn a_character_saved_while_writes_fail_is_written_later_and_read_back_meanwhile() {
        let (backend, queue, world) = world_in_memory();
        backend.cant_write.store(true, Ordering::Relaxed);
        let mut test = TestClient::of(world);
        test.send(&[ClientMessage::DevTeleport { x: 5, y: 6 }]);
        let world = test.leave();
        let key = SaveKey::Character(CharacterId::new("tester").unwrap());
        assert_eq!(backend.holds(&key), None);

        let test = TestClient::of(world);
        assert_eq!(test.client.world_position(), (5, 6));

        backend.cant_write.store(false, Ordering::Relaxed);
        assert!(queue.flush(Duration::from_secs(5)));
        assert!(backend.holds(&key).is_some());
    }

    #[test]
    fn a_site_that_cant_be_reached_is_refused_and_not_saved_over() {
        let (backend, queue, world) = world_in_memory();
        let id = SiteId::Dev {
            kind: PoiKind::Castle,
            seed: 1,
            ruin: 0,
        };
        let key = SaveKey::Site(world.seed.id(), id);
        let saved = SiteSave {
            version: 1,
            ..SiteSave::default()
        };
        world.storage.save_site(id, &saved);
        assert!(queue.flush(Duration::from_secs(5)));
        let before = backend.holds(&key);

        let mut test = TestClient::of(world);
        backend.cant_read.store(true, Ordering::Relaxed);
        let answers = test.send(&[ClientMessage::DevEnterSite {
            kind: PoiKind::Castle,
            seed: 1,
            level: 0,
            ruin: 0,
        }]);
        assert!(refused(&answers));

        // Asked for anyway, somewhere that can't refuse: blank, and not saved.
        test.world
            .site_mut(id, PoiKind::Castle)
            .levels
            .entry(9)
            .or_default();
        test.world.save_sites();
        assert!(queue.flush(Duration::from_secs(5)));
        assert_eq!(backend.holds(&key), before);

        // Back to normal: the real one again.
        backend.cant_read.store(false, Ordering::Relaxed);
        test.world.open_site(id, PoiKind::Castle).unwrap();
        assert!(test.world.site(id, PoiKind::Castle).levels.is_empty());
    }

    #[test]
    fn guests_come_back_by_their_token_and_the_newest_connection_wins() {
        let folder = crate::storage::tests::TestFolder::new();
        let seed = WorldSeed::new(1);
        let storage = crate::storage::Storage::in_folder(&folder.0, seed.id());
        let world = ServerWorld::open(seed, storage).unwrap();
        let mut connections = Connections::new(Accounts::new(SaveQueue::in_folder(&folder.0)));
        let (first, link) = local_pair();
        connections.add(link);
        first.send(&HELLO);
        first.send(&GUEST);
        let step = |n| ClientMessage::Step {
            map: 0,
            number: n,
            step: Step::new(1, 0).unwrap(),
        };
        let (connections, world) = tick(connections, world);
        assert!(matches!(
            answers(&first)[1],
            ServerMessage::Welcome { x: 0, .. }
        ));
        first.send(&step(1));
        let (mut connections, world) = tick(connections, world);
        answers(&first);

        // The same guest again (a reconnect): the old connection goes, and
        // the character comes back where it was.
        let (second, link) = local_pair();
        connections.add(link);
        second.send(&HELLO);
        second.send(&GUEST);
        let (connections, _world) = tick(connections, world);
        assert!(matches!(answers(&first)[..], [ServerMessage::Refused(_)]));
        assert!(matches!(
            answers(&second)[1],
            ServerMessage::Welcome { x: 1, .. }
        ));
        assert_eq!(connections.clients.len(), 1);
    }

    #[test]
    fn forgetting_a_site_on_the_server_puts_only_others_out_at_the_poi() {
        let mut connections = Connections::default();
        let (first, link) = local_pair();
        connections.add_playing(link, CharacterId::new("first").unwrap());
        let (second, link) = local_pair();
        connections.add_playing(link, CharacterId::new("second").unwrap());
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
        assert!(connections.clients[0].visit.is_some());
        assert!(matches!(answers(&first)[0], ServerMessage::EnterLevel(_)));
        assert!(connections.clients[1].visit.is_none());
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
        connections.add_playing(link, CharacterId::new("tester").unwrap());
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
        let mut connections = connections;
        let options = ServerOptions {
            dev_tools: true,
            exit_when_alone: false,
            pausable: false,
            guests: true,
        };
        assert!(answer(
            &mut connections.clients[0],
            ClientMessage::DevSetTime { minutes: 1200.0 },
            &mut world,
            &mut connections.accounts,
            &options
        ));
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
            action_ticks: 0,
        });
        let world = ServerWorld::open(seed, storage).unwrap();
        assert!((150.0..160.0).contains(&world.clock.minutes()));
        assert_eq!(world.hour, 2);
    }

    #[test]
    fn a_pausable_server_stops_its_clock_when_asked() {
        let mut test = TestClient::new();
        test.pausable = true;
        let answers = test.send(&[ClientMessage::SetPaused(true)]);
        let stopped = test.world.clock.minutes();
        assert_eq!(
            answers,
            vec![ServerMessage::Paused {
                paused: true,
                clock: stopped,
            }]
        );
        std::thread::sleep(std::time::Duration::from_millis(5));
        assert_eq!(test.world.clock.minutes(), stopped);

        // Pings are still answered, with the clock where it stopped.
        let answers = test.send(&[ClientMessage::Ping { sent: 1.5 }]);
        assert_eq!(
            answers,
            vec![ServerMessage::Pong {
                sent: 1.5,
                clock: stopped,
            }]
        );

        let answers = test.send(&[ClientMessage::SetPaused(false)]);
        assert!(matches!(
            answers[..],
            [ServerMessage::Paused { paused: false, .. }]
        ));
        std::thread::sleep(std::time::Duration::from_millis(5));
        assert!(test.world.clock.minutes() > stopped);
    }

    #[test]
    fn other_servers_never_pause() {
        // Dev tools being on makes no difference.
        let mut test = TestClient::new();
        let answers = test.send(&[ClientMessage::SetPaused(true)]);
        assert!(answers.is_empty());
        let before = test.world.clock.minutes();
        std::thread::sleep(std::time::Duration::from_millis(5));
        assert!(test.world.clock.minutes() > before);
        assert!(matches!(
            test.welcome[0],
            ServerMessage::Welcome {
                can_pause: false,
                ..
            }
        ));
    }

    #[test]
    fn messages_before_hello_are_refused() {
        let (end, server_link) = local_pair();
        let mut client = Client::new(server_link, CharacterId::new("x"));
        let mut world = ServerWorld::new(WorldSeed::new(1));
        let mut accounts = Accounts::default();
        let options = ServerOptions {
            dev_tools: false,
            exit_when_alone: false,
            pausable: false,
            guests: true,
        };
        let message = ClientMessage::UseExit;
        assert!(!answer(
            &mut client,
            message,
            &mut world,
            &mut accounts,
            &options
        ));
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
