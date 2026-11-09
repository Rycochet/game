//! Connected clients: reading their messages, answering them, and saving
//! their characters.

use bevy::prelude::*;
use shared::creature::CreatureKind;
use shared::energy::Energy;
use shared::id::Id;
use shared::item::Inventory;
use shared::poi::ChunkPos;
use shared::protocol::{CharacterId, ServerLink, ServerMessage};
use shared::sheet::{Appearance, Build, Feeling, Sheet};
use shared::wear::WornLook;
use shared::world::WorldId;
use std::collections::{BTreeMap, BTreeSet, VecDeque};

use crate::ServerWorlds;
use crate::accounts::{AccountId, Accounts};
use crate::ground::GroundTold;
use crate::sites::Visit;
use crate::storage::CharacterWorldSave;

mod answer_dev;
mod answer_items;
mod change_kind;
mod characters;

mod action;
mod action_bar;
mod answer;
mod client;
mod combat;
mod delete_character;
mod dev_skills;
pub(crate) mod difficulty;
mod give_items;
mod heal;
mod lightning;
mod login;
mod look_up;
mod messages;
mod moving;
mod new_character;
mod playtime;
mod roads;
mod role;
mod systems;
mod visited_gates;
mod vitals;
mod wearing;
mod welcome;

use action::Action;
pub(super) use answer::answer;
pub(crate) use combat::take_mob_blows;
pub(super) use lightning::tell_of_lightning;
pub(super) use login::finish_logins;
pub(crate) use messages::Sneak;
pub(super) use messages::handle_messages;
pub(crate) use moving::Arrival;
pub use playtime::IDLE_MINUTES;
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

    /// The embedded server's own player, already logged in to their account,
    /// to choose a character. Refused if saves can't be reached, as anyone else
    /// would be.
    pub fn add_single_player(&mut self, link: ServerLink) {
        match self.accounts.single_player() {
            Ok(account) => self.clients.push(Client::host(link, account)),
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
    pub(crate) fn send_others_away(&mut self, worlds: &mut ServerWorlds, message: &ServerMessage) {
        for client in self.clients.iter_mut().filter(|client| !client.host) {
            client.link.send(message);
            client.disconnect(worlds.of(client.world));
        }
        self.clients.retain(|client| !client.gone);
    }

    /// Disconnects everyone, saving them (the server is stopping).
    pub(crate) fn disconnect_all(&mut self, worlds: &mut ServerWorlds) {
        for client in &mut self.clients {
            client.disconnect(worlds.of(client.world));
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
    /// The embedded server's own player (connected without logging in): its
    /// Owner, who may open it to others.
    pub(crate) host: bool,
    /// Its account's role, once logged in (`role.rs`).
    held: Option<crate::roles::Held>,
    /// What the host last asked of opening the server to others, not yet done.
    pub(crate) asked_online: Option<Option<shared::protocol::OnlineSettings>>,
    pub(crate) phase: Phase,
    /// It said it can make a character (`Hello`), so a new one is made by it.
    makes_characters: bool,
    /// The account it logged in to (none on an embedded server).
    account: Option<AccountId>,
    /// The login it began on a provider's website, by its state (`oidc`).
    login_state: Option<String>,
    /// Its character: given when it connected, or decided by logging in.
    pub(crate) character_id: Option<CharacterId>,
    /// Its character's name, as others see it: set on joining.
    pub(crate) name: String,
    /// Whether it is to be disconnected (and was saved).
    gone: bool,
    /// The world its character is in, once joining has decided it. In the
    /// Hall, the world they came from.
    pub(crate) world: Option<WorldId>,
    /// The world it has asked to go to (dev tools), moved to once its
    /// messages this tick have been read.
    going_to: Option<(shared::world::WorldSeed, moving::Arrival)>,
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
    /// What each could be seen wearing when last told, to tell again on a change.
    pub(crate) seen_worn: BTreeMap<Id, Vec<WornLook>>,
    /// What the character is like.
    pub(crate) sheet: Sheet,
    /// Its action bar (`CharacterSave::action_bar`).
    action_bar: shared::ability::ActionBar,
    /// The World Gates it has gone through (`CharacterSave::gates`).
    gates: BTreeSet<u32>,
    /// What it was last told is on the ground under the player.
    pub(crate) ground_told: GroundTold,
    /// Its character's playtime, and when its player last did anything.
    pub(crate) activity: playtime::Activity,
    /// How much Health and Mana there can be, as the client was last told.
    vitals_told: Option<shared::vitals::Limits>,
    /// How much it can do now, and how fast that comes back.
    energy: Energy,
    speed: u32,
    /// What it has asked to do and hasn't had the energy for yet, in order.
    waiting: VecDeque<Action>,
    /// The ability it is winding up, and its attacks so far.
    fight: crate::combat::Fight,
    /// Its open practice windows, by skill and by kind of foe.
    learning: crate::combat::Learning,
    /// What a character being made here asked to be, for when they first join.
    starting_as: Option<(CreatureKind, Appearance, Build)>,
    /// Its character's difficulty: from `Hello`, making them or their save.
    pub(crate) difficulty: Option<shared::difficulty::Difficulty>,
    /// Killed on Ironman: deleted once the mobs' turn is done (`systems`).
    pub(crate) died_for_good: bool,
    /// What the host last asked the server to allow, not yet done.
    asked_difficulties: Option<shared::difficulty::Difficulties>,
    /// Whether `character` (or the visit) changed since it was saved.
    pub(crate) unsaved: bool,
    /// The site level the player is on, if any.
    pub(crate) visit: Option<Visit>,
    /// In the Hall of Worlds, where the client's `x` counts from
    /// (`ServerMessage::EnterHall`).
    pub(crate) hall_origin: i64,
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
    /// Items it has asked after (`LookUpItems`), answered once every client's
    /// messages this tick have been read.
    looking_up: Vec<Id>,
    /// Give Item's batches for other players, given once every message is read.
    dev_gifts: Vec<(Id, give_items::Batch)>,
}

#[cfg(test)]
mod action_bar_tests;
#[cfg(test)]
mod difficulty_tests;
#[cfg(test)]
mod idle_tests;
#[cfg(test)]
mod login_tests;
#[cfg(test)]
mod look_up_tests;
#[cfg(test)]
mod new_character_tests;
#[cfg(test)]
mod shapeshift_tests;
#[cfg(test)]
pub(crate) mod tests;
#[cfg(test)]
mod vitals_tests;
#[cfg(test)]
mod wearing_tests;
#[cfg(test)]
mod worlds_tests;
