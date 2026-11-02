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
    world.save_mobs();
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
            client.send_inventory()
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
pub(crate) mod tests {
    /// The character a `TestClient` plays.
    fn tester() -> CharacterId {
        CharacterId(Id::for_test(1000))
    }

    use std::sync::Arc;
    use std::sync::atomic::Ordering;
    use std::time::Duration;

    use shared::item::{Favour, Holder, Rotation};
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
                client: Client::new(server_link, Some(tester())),
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
            ground::sync(std::slice::from_mut(&mut self.client), &mut self.world);
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
            quality: Quality::Rare,
            set: false,
            favour: None,
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
    fn moves_between_containers_take_time_and_sorting_one_doesnt() {
        let mut test = TestClient::new();
        test.time_passes = false;
        let answers = test.send(&[give(ItemKind::Pouch), give(ItemKind::Coin)]);
        let carried = inventory(&answers);
        let (pouch, coin) = (carried.pockets[0].item.id, carried.pockets[1].item.id);
        let tick = test.world.action_clock.ticks();
        let energy = test.client.energy.at(tick, test.client.speed);
        test.send(&[ClientMessage::MoveItem {
            item: coin,
            to: Holder::Pockets,
            x: 5,
            y: 3,
            rotation: Rotation::None,
        }]);
        assert_eq!(test.client.energy.at(tick, test.client.speed), energy);
        test.send(&[ClientMessage::MoveItem {
            item: coin,
            to: Holder::Inside(pouch),
            x: 0,
            y: 0,
            rotation: Rotation::None,
        }]);
        assert!(test.client.energy.at(tick, test.client.speed) < energy);
    }

    #[test]
    fn destroying_takes_what_is_carried_and_only_that() {
        let mut test = TestClient::new();
        let answers = test.send(&[give(ItemKind::Coin)]);
        let coin = inventory(&answers).pockets[0].item.id;
        let destroy = ClientMessage::BinItem { item: coin };
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
    fn new_items_have_the_quality_asked_for_and_the_time_they_were_made() {
        let mut test = TestClient::new();
        // A dagger, as a long enough sword wouldn't fit in the pockets.
        let answers = test.send(&[give(ItemKind::Dagger), give(ItemKind::Coin)]);
        let carried = inventory(&answers);
        let (sword, coin) = (&carried.pockets[0].item, &carried.pockets[1].item);
        assert_eq!(sword.quality, Some(Quality::Rare));
        assert_eq!(coin.quality, None, "a coin has no quality");
        let made = sword.made_at.expect("new things know when they were made");
        assert_eq!(made.world, test.world.seed.id());
        assert_eq!(sword.name, None);
    }

    #[test]
    fn only_things_with_quality_can_be_part_of_a_set() {
        let mut test = TestClient::new();
        let give = |kind| ClientMessage::DevGiveItem {
            kind,
            size: None,
            magical: false,
            quality: Quality::Poor,
            set: true,
            favour: None,
        };
        test.send(&[give(ItemKind::Hat), give(ItemKind::Apple)]);
        let set_of = |kind| {
            let pockets = test.client.inventory.pockets.iter();
            let mut pockets = pockets.filter(|placed| placed.item.kind == kind);
            pockets.next_back().unwrap().item.set
        };
        assert!(set_of(ItemKind::Hat));
        assert!(!set_of(ItemKind::Apple));
    }

    #[test]
    fn a_new_ring_has_a_form_and_fits_its_wearer_and_a_new_gem_rolls_its_size() {
        let mut test = TestClient::new();
        let give = |kind| ClientMessage::DevGiveItem {
            kind,
            size: None,
            magical: false,
            quality: Quality::Rare,
            set: false,
            favour: None,
        };
        let carried = inventory(&test.send(&[give(ItemKind::Ring)]));
        let ring = &carried.pockets[0].item;
        assert!(ring.form.is_some());
        assert_eq!(ring.made.size, carried.creature.def().size);
        let carried = inventory(&test.send(&[give(ItemKind::Gem)]));
        assert!(
            carried
                .pockets
                .iter()
                .any(|placed| { placed.item.kind == ItemKind::Gem && placed.item.form.is_some() })
        );
    }

    #[test]
    fn artifacts_are_always_named() {
        let mut test = TestClient::new();
        let artifact = ClientMessage::DevGiveItem {
            kind: ItemKind::Sword,
            size: None,
            magical: false,
            quality: Quality::Artifact,
            set: false,
            favour: None,
        };
        let carried = inventory(&test.send(&[artifact]));
        let name = carried.pockets[0].item.name();
        assert!(name.ends_with(" Sword") && name != "Sword", "{name}");
    }

    #[test]
    fn items_saved_before_quality_still_load() {
        let saved =
            ron::to_string(&Item::new(shared::id::Id::for_test(1), ItemKind::Sword)).unwrap();
        let old = saved.replace(",quality:None,made_at:None,name:None", "");
        assert_ne!(old, saved);
        let item: Item = ron::from_str(&old).unwrap();
        assert_eq!((item.quality, item.made_at), (None, None));
    }

    #[test]
    fn items_are_made_for_the_wearer_or_the_size_asked_for() {
        let mut test = TestClient::new();
        test.send(&[ClientMessage::DevSetCreature(CreatureKind::Giant)]);
        test.send(&[ClientMessage::DevTruesight(true)]);
        let answers = test.send(&[give(ItemKind::Gloves)]);
        let gloves = inventory(&answers).pockets.last().unwrap().item.made;
        assert_eq!(gloves.size, shared::creature::Size::Giant);
        assert_eq!(gloves.fingers, Some(10));
        let large = |kind| ClientMessage::DevGiveItem {
            kind,
            size: Some(shared::creature::Size::Large),
            magical: true,
            quality: Quality::Epic,
            set: false,
            favour: None,
        };
        let mut made = |kind| {
            let answers = test.send(&[large(kind)]);
            inventory(&answers).pockets.last().unwrap().item.made
        };
        let hat = made(ItemKind::Hat);
        assert_eq!(hat.size, shared::creature::Size::Large);
        assert!(hat.magical);
        // Not something that comes in sizes, whoever asks or carries it.
        let coin = made(ItemKind::Coin);
        assert_eq!(coin.size, shared::creature::Size::Medium);
    }

    /// Something new in the pockets, with these secrets, and its id.
    fn given(
        test: &mut TestClient,
        kind: ItemKind,
        size: Option<shared::creature::Size>,
        magical: bool,
        favour: Option<Favour>,
    ) -> Id {
        let answers = test.send(&[ClientMessage::DevGiveItem {
            kind,
            size,
            magical,
            quality: Quality::Rare,
            set: false,
            favour,
        }]);
        inventory(&answers).pockets.last().unwrap().item.id
    }

    fn secretly_cursed_hat() -> (TestClient, Id) {
        let mut test = TestClient::new();
        let hat = given(&mut test, ItemKind::Hat, None, true, Some(Favour::Cursed));
        test.send(&[ClientMessage::WearItem { item: hat }]);
        (test, hat)
    }

    #[test]
    fn secrets_are_only_sent_to_truesight_or_once_found_out() {
        let (mut test, hat) = secretly_cursed_hat();
        assert!(test.client.inventory.find(hat).unwrap().is_magical());
        let shoes = given(&mut test, ItemKind::Shoes, None, false, None);
        let answers = test.send(&[ClientMessage::BinItem { item: shoes }]);
        let seen = inventory(&answers);
        let item = seen.find(hat).unwrap();
        assert!(!item.made.magical);
        assert_eq!(item.favour, None);

        let seen = inventory(&test.send(&[ClientMessage::DevTruesight(true)]));
        let item = seen.find(hat).unwrap();
        assert!(item.made.magical);
        assert_eq!(item.favour, Some(Favour::Cursed));
        assert!(
            !item.known.magic && !item.known.favour,
            "seen, not found out"
        );

        let seen = inventory(&test.send(&[ClientMessage::DevTruesight(false)]));
        assert_eq!(seen.find(hat).unwrap().favour, None);
    }

    #[test]
    fn trying_to_take_off_a_curse_fails_and_gives_it_away() {
        let attempts = |hat| {
            [
                ClientMessage::MoveItem {
                    item: hat,
                    to: Holder::Pockets,
                    x: 0,
                    y: 0,
                    rotation: Rotation::None,
                },
                ClientMessage::BinItem { item: hat },
                ClientMessage::GiveItem {
                    item: hat,
                    to: Id::for_test(9),
                },
            ]
        };
        for index in 0..3 {
            let (mut test, hat) = secretly_cursed_hat();
            let answers = test.send(&[attempts(hat)[index].clone()]);
            let seen = inventory(&answers);
            assert!(seen.is_worn(hat), "attempt {index}");
            let item = seen.find(hat).unwrap();
            assert_eq!(item.favour, Some(Favour::Cursed), "attempt {index}");
            assert!(!item.made.magical, "only the curse is found out");
        }
    }

    #[test]
    fn in_god_mode_a_curse_comes_off_and_stays_secret() {
        let (mut test, hat) = secretly_cursed_hat();
        test.send(&[ClientMessage::DevGodMode(true)]);
        let answers = test.send(&[ClientMessage::BinItem { item: hat }]);
        let seen = inventory(&answers);
        assert!(!seen.is_worn(hat));
        assert_eq!(seen.bin.unwrap().favour, None);
    }

    #[test]
    fn a_blessing_lets_its_wearer_take_it_off_and_stays_secret() {
        let mut test = TestClient::new();
        let hat = given(&mut test, ItemKind::Hat, None, false, Some(Favour::Blessed));
        test.send(&[ClientMessage::WearItem { item: hat }]);
        let seen = inventory(&test.send(&[ClientMessage::BinItem { item: hat }]));
        assert!(seen.in_bin(hat));
        assert_eq!(seen.bin.unwrap().favour, None);
    }

    #[test]
    fn wrong_sized_magic_gives_itself_away_by_fitting() {
        use shared::creature::Size;
        let mut test = TestClient::new();
        let own = given(&mut test, ItemKind::Hat, None, true, None);
        let answers = test.send(&[ClientMessage::WearItem { item: own }]);
        assert!(!inventory(&answers).find(own).unwrap().made.magical);

        // Too small for a medium wearer, were it not magical. Rare reaches 2.
        let mut test = TestClient::new();
        let tiny = given(&mut test, ItemKind::Hat, Some(Size::Tiny), true, None);
        let answers = test.send(&[ClientMessage::WearItem { item: tiny }]);
        let seen = inventory(&answers);
        assert!(seen.is_worn(tiny));
        assert!(seen.find(tiny).unwrap().made.magical);

        let mut test = TestClient::new();
        test.send(&[ClientMessage::DevSetCreature(CreatureKind::Giant)]);
        let far = given(&mut test, ItemKind::Hat, Some(Size::Tiny), true, None);
        let answers = test.send(&[ClientMessage::WearItem { item: far }]);
        assert!(!inventory(&answers).is_worn(far), "past its reach");
    }

    #[test]
    fn asking_for_magic_on_something_plain_raises_its_quality() {
        let mut test = TestClient::new();
        test.send(&[ClientMessage::DevTruesight(true)]);
        let answers = test.send(&[ClientMessage::DevGiveItem {
            kind: ItemKind::Hat,
            size: None,
            magical: true,
            quality: Quality::Common,
            set: false,
            favour: None,
        }]);
        let hat = inventory(&answers).pockets.last().unwrap().item.clone();
        assert!(hat.is_magical());
        assert_eq!(hat.quality, Some(Quality::Uncommon));
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
        let theirs = list[0].id;

        // Someone else's can't be chosen.
        let (other, _) = logged_in_to_account(&mut connections, "someone else");
        other.send(&ClientMessage::ChooseCharacter { id: theirs });
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
        let key = SaveKey::Character(tester());
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
        let key = SaveKey::Character(tester());
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
        connections.add_playing(link, CharacterId(Id::for_test(1)));
        let (second, link) = local_pair();
        connections.add_playing(link, CharacterId(Id::for_test(2)));
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
        connections.add_playing(link, tester());
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
        let mut client = Client::new(server_link, Some(tester()));
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
