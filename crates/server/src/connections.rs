//! Connected clients: reading their messages, answering them, and saving
//! their characters.

use std::collections::{BTreeMap, BTreeSet, VecDeque};

use bevy::prelude::*;
use shared::creature::CreatureKind;
use shared::energy::{
    CHANGE_MAP_COST, Energy, NORMAL_SPEED, dressing_cost, handling_cost, site_step_cost,
};
use shared::id::Id;
use shared::item::{Inventory, Item, ItemKind, Made, MadeAt, Quality};
use shared::item_form::give_form;
use shared::movement::{Step, world_step_cost};
use shared::poi::ChunkPos;
use shared::protocol::{
    CharacterId, ClientMessage, LoginMethod, PROTOCOL_VERSION, ServerLink, ServerMessage,
};
use shared::sheet::{Appearance, Feeling, Sheet};
use shared::wear::Remover;
use shared::world::{MINUTES_PER_HOUR, WorldId};

use crate::accounts::{AccountId, Accounts, LoginKey, MAX_CHARACTERS};
use crate::ground::{self, GroundAction, GroundTold};
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

    /// The embedded server's one player, with their character. Refused if saves
    /// can't be reached, as anyone else would be.
    pub fn add_single_player(&mut self, link: ServerLink) {
        match self.accounts.single_player() {
            Ok(character) => self.add_playing(link, character),
            Err(error) => {
                let mut client = Client::new(link, None);
                client.refuse_for_saves(&error);
                client.gone = true;
                self.clients.push(client);
            }
        }
    }

    pub fn is_empty(&self) -> bool {
        self.clients.is_empty()
    }

    /// Everyone connected, for the debug API.
    #[cfg(feature = "debug-api")]
    pub(crate) fn clients(&self) -> &[Client] {
        &self.clients
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
    OpenTrade { with: Id },
    Ground(GroundAction),
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
    pub(crate) character_id: Option<CharacterId>,
    /// Whether it is to be disconnected (and was saved).
    gone: bool,
    /// What the character knows of this world and where they are in it: the
    /// pois and cells they know (`visit` is filled in on saving).
    pub(crate) character: CharacterWorldSave,
    /// What the character knows of other worlds, kept for saving.
    other_worlds: BTreeMap<WorldId, CharacterWorldSave>,
    /// What the character carries.
    pub(crate) inventory: Inventory,
    /// Dev tools: it is told of every mob on its level, seen or not.
    pub(crate) detect_monsters: bool,
    /// Dev tools: mobs take no notice of it, and it can walk through them.
    pub(crate) god_mode: bool,
    /// Dev tools: it is told items' secrets, without them being found out.
    pub(crate) truesight: bool,
    /// Who the client has been told is in sight, and where.
    pub(crate) seen_entities: BTreeMap<Id, (i64, i64)>,
    /// How the client has been told they feel.
    pub(crate) seen_feelings: BTreeMap<Id, Feeling>,
    /// What the character is like.
    pub(crate) sheet: Sheet,
    /// What it was last told is on the ground under the player.
    pub(crate) ground_told: GroundTold,
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
            detect_monsters: false,
            god_mode: false,
            truesight: false,
            seen_entities: BTreeMap::new(),
            seen_feelings: BTreeMap::new(),
            sheet: Sheet::default(),
            ground_told: GroundTold::default(),
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
        self.seen_feelings.clear();
        self.ground_told = GroundTold::default();
    }

    /// What it carries has changed, which can change how fast it goes.
    pub(crate) fn update_speed(&mut self, tick: u64) {
        self.energy.settle(tick, self.speed);
        self.speed = self.sheet.speed(&self.inventory);
    }

    /// Tells the client what it carries, as far as it knows.
    pub(crate) fn send_inventory(&mut self) -> bool {
        let mut seen = self.inventory.clone();
        if !self.truesight {
            seen.hide_secrets();
        }
        self.link.send(&ServerMessage::Inventory(seen))
    }

    /// These items, as far as the client knows.
    pub(crate) fn as_seen(&self, mut items: Vec<Item>) -> Vec<Item> {
        if !self.truesight {
            items.iter_mut().for_each(Item::hide_secrets);
        }
        items
    }

    /// Whether the player's own attempt to take this off gets anywhere. A curse
    /// holding it is found out by trying, unless something on top stopped the attempt.
    pub(crate) fn may_take_off(&mut self, item: shared::id::Id) -> bool {
        if self.god_mode
            || self.inventory.covered_by(item).is_some()
            || self.inventory.may_remove(item, Remover::Wearer)
        {
            return true;
        }
        self.inventory.favour_found_out(item);
        self.unsaved = true;
        false
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
    /// What others know this character by: their character's id. Nil until there
    /// is a character, when nobody can see them anyway.
    pub(crate) fn entity_id(&self) -> Id {
        self.character_id.map_or(Id(uuid::Uuid::nil()), |id| id.0)
    }

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
            Action::OpenTrade { with } => (sites::open_trade(client, world, with), 0),
            Action::Ground(action) => ground::act(client, world, action),
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
            let place = (visit.id(), visit.index());
            // Someone who won't move aside is in the way; anyone else swaps places.
            if let Some(to) = visit.landing(step) {
                let kind = client.inventory.creature;
                let character = client.character_id.as_ref();
                if !client.god_mode && world.enemy_at(place, to, kind, character) {
                    info!("Refused a step onto an enemy, at {to:?}");
                    return (true, 0);
                }
            }
            let stayed = sites::stepped(client, step);
            let to = client.visit.as_ref().map_or(from, Visit::position);
            if to == from {
                return (stayed, 0);
            }
            world.swap_mob(place, to, from);
            (stayed, site_step_cost(from, to))
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
    let account = clients[i].account;
    let character = clients[i].character_id;
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
    save_all(&mut connections, &mut world);
}

/// Queues everything changed to be saved: mobs, characters and sites.
pub(crate) fn save_all(connections: &mut Connections, world: &mut ServerWorld) {
    world.save_mobs();
    for client in &mut connections.clients {
        client.save(world);
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

/// Every tick: tells everyone what has come to be on the ground under them.
pub(crate) fn sync_ground(mut connections: ResMut<Connections>, mut world: ResMut<ServerWorld>) {
    ground::sync(&mut connections.clients, &mut world);
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
            let Some(account) = client.account.filter(|_| client.phase == Phase::Choosing) else {
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
            let Some(account) = client.account.filter(|_| client.phase == Phase::Choosing) else {
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
        ClientMessage::OpenTrade { with } => {
            client.waiting.push_back(Action::OpenTrade { with });
            true
        }
        ClientMessage::DropItem { item } => {
            let action = Action::Ground(GroundAction::Drop { item });
            client.waiting.push_back(action);
            true
        }
        ClientMessage::TakeFromGround {
            item,
            to,
            x,
            y,
            rotation,
        } => {
            client.waiting.push_back(Action::Ground(GroundAction::Take {
                item,
                to,
                x,
                y,
                rotation,
            }));
            true
        }
        ClientMessage::WearFromGround { item } => {
            let action = Action::Ground(GroundAction::Wear { item });
            client.waiting.push_back(action);
            true
        }
        ClientMessage::BinFromGround { item } => {
            let action = Action::Ground(GroundAction::Bin { item });
            client.waiting.push_back(action);
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
            let cost = client.inventory.move_cost(item, to);
            let tick = world.action_clock.ticks();
            if client.may_take_off(item) {
                match client.inventory.move_item(item, to, x, y, rotation) {
                    Ok(()) => {
                        client.unsaved = true;
                        client.energy.spend(tick, client.speed, cost);
                    }
                    Err(why) => warn!("Refused moving item {item:?} to {to:?}: {why:?}"),
                }
            }
            client.update_speed(world.action_clock.ticks());
            client.send_inventory()
        }
        ClientMessage::GiveItem { item, to } => {
            let mut stock = None;
            let cost = client.inventory.put_down_cost(item);
            let given = if !client.may_take_off(item) {
                Err("it won't come off")
            } else if sites::keeper_building(client, world, to).is_some() {
                sites::give_to_keeper(client, world, item, to).map(|items| stock = Some(items))
            } else {
                crate::entities::give(client, world, item, to)
            };
            match given {
                Ok(()) => {
                    client.unsaved = true;
                    let tick = world.action_clock.ticks();
                    client.energy.spend(tick, client.speed, cost);
                }
                Err(why) => warn!("Refused giving {item:?} to {to:?}: {why}"),
            }
            client.update_speed(world.action_clock.ticks());
            let sent = client.send_inventory();
            match stock {
                Some(items) => {
                    let items = client.as_seen(items);
                    sent && client
                        .link
                        .send(&ServerMessage::TradeStock { with: to, items })
                }
                None => sent,
            }
        }
        ClientMessage::TakeItem {
            with,
            item,
            to,
            x,
            y,
            rotation,
        } => {
            match sites::take_from_keeper(client, world, with, item, (to, x, y, rotation)) {
                Ok(()) => {
                    client.unsaved = true;
                    let grams = client.inventory.find(item).map_or(0, Item::total_weight);
                    let tick = world.action_clock.ticks();
                    client
                        .energy
                        .spend(tick, client.speed, handling_cost(grams));
                }
                Err(why) => warn!("Refused taking {item:?} from {with:?}: {why}"),
            }
            client.update_speed(world.action_clock.ticks());
            let stock = sites::keeper_stock(client, world, with);
            let sent = client.send_inventory();
            match stock {
                Some(items) => {
                    let items = client.as_seen(items);
                    sent && client.link.send(&ServerMessage::TradeStock { with, items })
                }
                None => sent,
            }
        }
        ClientMessage::WearItem { item } => {
            let tick = world.action_clock.ticks();
            match client.inventory.wear(item) {
                Ok(_) => {
                    client.unsaved = true;
                    client.inventory.magic_found_out(item);
                    client.spend_dressing(item, tick);
                }
                Err(why) => warn!("Refused wearing {item:?}: {why:?}"),
            }
            client.update_speed(world.action_clock.ticks());
            client.send_inventory()
        }
        ClientMessage::BinItem { item } => {
            if !client.may_take_off(item) {
                // Nothing more to say: it stays on.
            } else if client.inventory.to_bin(item) {
                client.unsaved = true;
            } else {
                warn!("Refused throwing away {item:?}");
            }
            client.update_speed(world.action_clock.ticks());
            client.send_inventory()
        }
        // Dev tools are only for the embedded server; elsewhere a stray key
        // or click is ignored rather than ending the game.
        _ if message.is_dev_tool() && !options.dev_tools => {
            warn!("A client asked for dev tools ({message:?}), which this server doesn't allow");
            true
        }
        ClientMessage::DevTeleport { x, y } => match &client.visit {
            Some(visit) => {
                let place = (visit.id(), visit.index());
                let stayed = sites::teleport(client, x, y);
                if let Some(visit) = &client.visit {
                    world.move_mob_aside(place, visit.position());
                }
                stayed
            }
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
            quality,
            set,
            favour,
        } => {
            // Asking for magic on what is too plain for it would quietly give none.
            let quality = if magical && kind.def().wear.is_some() {
                quality.max(Quality::Uncommon)
            } else {
                quality
            };
            let mut new = made_for(
                &client.inventory,
                kind,
                size,
                magical,
                quality,
                world.made_now(),
            );
            new.set = set && new.quality.is_some();
            new.made.magical = magical && new.can_be_magical();
            new.favour = favour.filter(|_| kind.def().wear.is_some());
            match client.inventory.give(new) {
                Ok(()) => client.unsaved = true,
                Err(item) => info!("No room for a new {:?}", item.kind),
            }
            client.update_speed(world.action_clock.ticks());
            client.send_inventory()
        }
        ClientMessage::DevDetectMonsters(on) => {
            client.detect_monsters = on;
            true
        }
        ClientMessage::DevTruesight(on) => {
            client.truesight = on;
            client.ground_told.again = true;
            client.send_inventory() && sites::truesight_changed(client, world)
        }
        ClientMessage::DevGodMode(on) => {
            client.god_mode = on;
            true
        }
        ClientMessage::DevKillMob { mob } => {
            if let Err(why) = crate::entities::kill(client, world, mob) {
                warn!("Refused to kill {mob:?}: {why}");
            }
            true
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
                    client.send_inventory()
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
/// Kinds that don't come in sizes are `Medium`, whatever is asked for.
pub(crate) fn made_for(
    inventory: &Inventory,
    kind: ItemKind,
    size: Option<shared::creature::Size>,
    magical: bool,
    quality: Quality,
    made_at: MadeAt,
) -> Item {
    let body = inventory.creature.def();
    let size = size.filter(|_| kind.comes_in_sizes());
    // What is worn is made to fit the wearer unless told otherwise.
    let size = size.or(kind.def().wear.map(|_| body.size));
    let id = crate::ids::new_id();
    let quality = kind.def().has_quality.then_some(quality);
    let name = (quality == Some(Quality::Artifact))
        .then(|| shared::item::artifact_name(&mut id.seed().rng(), kind));
    let mut item = Item {
        made: Made::for_body(kind, size.unwrap_or_default(), body.fingers(), magical),
        quality,
        made_at: Some(made_at),
        name,
        ..Item::new(id, kind)
    };
    give_form(&mut item, size, &mut crate::ids::new_id);
    item
}

/// What a joining character is like: as saved; or as they asked when being made;
/// or if they didn't ask, plain for their kind.
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

/// A new character's clothes.
fn give_first_clothes(inventory: &mut Inventory, made_at: MadeAt) {
    if std::mem::replace(&mut inventory.outfitted, true) {
        return;
    }
    for kind in [ItemKind::TShirt, ItemKind::Trousers, ItemKind::Shoes] {
        // Straight on, as the pockets may be full. Only for bodies they fit: other
        // kinds of creature start with none.
        if inventory.creature == shared::creature::CreatureKind::Human {
            let item = made_for(inventory, kind, None, false, Quality::Common, made_at);
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
            give_first_clothes(&mut client.inventory, world.made_now());
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
        || !client.send_inventory()
    {
        return false;
    }
    match client.character.visit.take() {
        Some(visit) => sites::resume(client, world, visit),
        None => world_map::look_around(client, world),
    }
}

#[cfg(test)]
pub(crate) mod tests;
