//! Connected clients: reading their messages, answering them, and saving
//! their characters.

use bevy::prelude::*;
use shared::creature::CreatureKind;
use shared::energy::Energy;
use shared::id::Id;
use shared::item::Inventory;
use shared::movement::Step;
use shared::poi::ChunkPos;
use shared::protocol::{CharacterId, ServerLink, ServerMessage};
use shared::sheet::{Appearance, Feeling, Sheet};
use shared::world::WorldId;
use std::collections::{BTreeMap, BTreeSet, VecDeque};

use crate::ServerWorld;
use crate::accounts::{AccountId, Accounts};
use crate::ground::{GroundAction, GroundTold};
use crate::sites::Visit;
use crate::storage::CharacterWorldSave;

mod answer_dev;
mod answer_items;

// Split into files (`cargo xtask lint` keeps files small).
mod answer;
mod client;
mod messages;
mod roads;
mod systems;
mod welcome;

pub(super) use answer::answer;
pub(super) use messages::handle_messages;
pub(super) use roads::send_roads;
#[cfg(feature = "debug-api")]
pub(super) use systems::save_all;
pub(super) use systems::{run_mobs, save_changes, sync_entities, sync_ground, tell_the_time};
pub(super) use welcome::made_for;
use welcome::welcome;

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

    /// What the host has asked of opening the server to others, if anything.
    pub(crate) fn take_online_request(
        &mut self,
    ) -> Option<Option<shared::protocol::OnlineSettings>> {
        let host = self.clients.iter_mut().find(|client| client.host)?;
        host.asked_online.take()
    }

    /// Tells the host (the embedded server's own player), if connected.
    pub(crate) fn tell_host(&self, message: &ServerMessage) {
        for client in self
            .clients
            .iter()
            .filter(|client| client.host && !client.gone)
        {
            client.link.send(message);
        }
    }

    /// Tells everyone but the host this, and disconnects them, saving them
    /// (the server is closing to others).
    pub(crate) fn send_others_away(&mut self, world: &mut ServerWorld, message: &ServerMessage) {
        for client in self.clients.iter_mut().filter(|client| !client.host) {
            client.link.send(message);
            client.disconnect(world);
        }
        self.clients.retain(|client| !client.gone);
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
#[derive(Clone, Debug)]
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
    /// The embedded server's own player (connected without logging in), who
    /// keeps dev tools and pausing when others join, and may open it to them.
    pub(crate) host: bool,
    /// What the host last asked of opening the server to others, not yet done.
    pub(crate) asked_online: Option<Option<shared::protocol::OnlineSettings>>,
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
    /// The roads it has been sent on this connection.
    pub(crate) sent_roads: BTreeSet<shared::road::RoadLink>,
    /// Every poi is shown, whether known or not: from the start while
    /// `ALL_POIS_KNOWN`, otherwise by developer tools.
    pub(crate) reveal_pois: bool,
    /// How many maps the player has been put on since `Welcome` (see
    /// `ClientMessage::Step`).
    map: u32,
    /// The last step dealt with this tick, to send `Position` for.
    last_step: Option<u32>,
}

#[cfg(test)]
pub(crate) mod tests;
