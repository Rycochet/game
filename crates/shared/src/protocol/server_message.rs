//! What the server sends a client.

use serde::{Deserialize, Serialize};

use super::level::{LevelChunk, LevelInfo, RememberedLevel};
use super::login::{CharacterSummary, LoginMethod};
use super::{WorldChunk, WorldInfo};
use crate::ground::Load;
use crate::id::Id;
use crate::item::{Inventory, Item};
use crate::mob::EntityLook;
use crate::sheet::Sheet;

/// From the server to a client.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum ServerMessage {
    /// The answer to `Hello`: the ways to log in here.
    LoginOptions { methods: Vec<LoginMethod> },
    /// The account's characters, to choose from or add to (if `can_create`).
    Characters {
        list: Vec<CharacterSummary>,
        can_create: bool,
    },
    /// What someone the player is trading with has to trade.
    TradeStock { with: Id, items: Vec<Item> },
    /// What is on the ground under the player, when they arrive on it or it changes,
    /// and only for someone standing there. Empty once there is nothing.
    GroundItems { x: i64, y: i64, items: Vec<Item> },
    /// There is a bag on the ground here, looking this full: seen just now, or what is
    /// remembered of it, after arriving on a level. Bags out of sight aren't kept up to
    /// date, so they can be wrong until they are next seen.
    GroundSeen { x: i64, y: i64, load: Load },
    /// There is no bag here after all, as just seen.
    GroundGone { x: i64, y: i64 },
    /// Who keeps each shop of a town (`Buildings` comes first), by `BuildingLook::id`.
    /// Never on the map: they stand behind the door.
    Keepers(Vec<(u16, Id)>),
    /// The player is in the game: the world they are in, its clock (in game
    /// hours since it began) and snow line, and where they are on its world
    /// map (where they left off). If they were in a site, `EnterLevel` follows.
    Welcome {
        world: WorldInfo,
        clock: f64,
        snow_line: u32,
        x: i64,
        y: i64,
        /// Whether this server accepts developer tools messages.
        dev_tools: bool,
        /// Whether this server pauses when asked: only single player's.
        can_pause: bool,
    },
    /// Everything the client knows of the world map is forgotten; the chunks
    /// it may know again follow (`DevForgetMap`).
    ForgetWorldMap,
    /// The snow line has moved: above it everything is frozen, and ice can be
    /// walked on.
    SnowLine(u32),
    /// A new whole hour has begun on the clock (`% 24` is the time of day).
    /// The tide, and so where can be walked on the world map, follows it.
    Hour(u32),
    /// The world's clock has been moved: game minutes since the world began.
    Clock(f64),
    /// The world's clock has stopped or started again, at this many game
    /// minutes. While stopped, the clock in `Pong` says nothing new.
    Paused { paused: bool, clock: f64 },
    /// The answer to `Ping`: its `sent`, and the world's clock (game minutes)
    /// when answering.
    Pong { sent: f64, clock: f64 },
    /// The server won't talk to this client, and why.
    Refused(String),
    /// A world map chunk, as this player knows it.
    WorldChunk(WorldChunk),
    /// The player is now on this site level; its cells follow.
    EnterLevel(LevelInfo),
    /// Cells of the current level.
    LevelCells(LevelChunk),
    /// Another level of this site that the player knows, sent after
    /// `EnterLevel` on coming into the site.
    RememberedLevel(RememberedLevel),
    /// A town's buildings, sent after `EnterLevel` and before its cells:
    /// players know a town's layout from the start.
    Buildings(Vec<crate::site::BuildingLook>),
    /// A farm's fields, sent after `EnterLevel` on its ground floor: what is
    /// needed to choose what grows in them.
    Fields(Vec<crate::site::FieldLook>),
    /// The player is on the world map, here: back out of a site, or sent
    /// back to the start by dev tools.
    OnWorldMap { x: i64, y: i64 },
    /// Developer tools: the player has jumped to here, on the map they are
    /// on (`DevTeleport`).
    Teleported { x: i64, y: i64 },
    /// Where the server has the player on map `map` (see
    /// `ClientMessage::Step`) once it has dealt with step `step`: the client
    /// adds the steps it took since, and moves the player there if it shows
    /// them elsewhere (a step the server refused). Sent once a tick, when
    /// steps have arrived.
    Position { map: u32, step: u32, x: i64, y: i64 },
    /// Everything the character carries, as the server has it: after Welcome,
    /// and whenever it is asked to change, so a move it refused is put right.
    Inventory(Inventory),
    /// What the character is like, for them to see: after `Welcome`, and when it
    /// changes. Nobody else is told this.
    Sheet(Sheet),
    /// Something has come into sight: a mob or another player. The map changing
    /// (`EnterLevel`, `OnWorldMap`, `Teleported`) forgets everything seen before.
    EntitySeen(EntityLook),
    /// Something in sight has moved.
    EntityMoved { id: Id, x: i64, y: i64 },
    /// Something has gone out of sight, or away.
    EntityGone { id: Id },
}
