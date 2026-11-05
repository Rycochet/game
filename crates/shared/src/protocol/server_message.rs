//! What the server sends a client.

use serde::{Deserialize, Serialize};

use super::OnlineState;
use super::level::{LevelChunk, LevelInfo, RememberedLevel};
use super::login::{CharacterSummary, LoginMethod};
use super::{WorldChunk, WorldInfo};
use crate::ground::Load;
use crate::id::Id;
use crate::item::{Inventory, Item, StackLimits};
use crate::mob::EntityLook;
use crate::road::Road;
use crate::sheet::Sheet;
use crate::world::WorldTime;

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
    /// The player is in the game: the world they are in, its time (worked out
    /// from the server's clock, which reads `server_clock` seconds now) and snow
    /// line, and where they are on its world map (where they left off). If they
    /// were in a site, `EnterLevel` follows.
    Welcome {
        world: WorldInfo,
        time: WorldTime,
        server_clock: f64,
        snow_line: u32,
        x: i64,
        y: i64,
        /// Whether this server accepts developer tools messages.
        dev_tools: bool,
        /// Whether this server pauses when asked: only single player's.
        can_pause: bool,
        /// How big stacks of items may get here.
        stacks: StackLimits,
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
    /// The world's time has been moved, or now runs at another rate.
    WorldClock(WorldTime),
    /// The server's clock has stopped or started again, at this many seconds,
    /// and every world's time with it. While stopped, the clock in `Pong` says
    /// nothing new.
    Paused { paused: bool, clock: f64 },
    /// The answer to `Ping`: its `sent`, and the server's clock (seconds) when
    /// answering, which every world's time is worked out from.
    Pong { sent: f64, clock: f64 },
    /// The server won't talk to this client, and why.
    Refused(String),
    /// Whether the server is open to other players, the answer to `SetOnline`.
    Online(OnlineState),
    /// The server is closing to other players on purpose (its host turned Online
    /// off), so this connection is ending and isn't worth trying again: the
    /// server's name.
    GoingOffline(String),
    /// A world map chunk, as this player knows it.
    WorldChunk(WorldChunk),
    /// A road between two towns, sent once it is worked out to anyone near
    /// enough to see any of it (towns are seen by all, so it gives nothing
    /// away). Sent once per connection; the client keeps it.
    Road(Road),
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
    /// The player is now in the Hall of Worlds (`shared::hall`), here. The
    /// client works out its cells itself. Positions in the Hall (this, and
    /// `Position`, `Teleported`, entities, `DevTeleport`) are counted from
    /// `origin`, where the Hall's `x` is billions, more than drawing can
    /// place: the Hall's own `x` is `x + origin`. `origin` is a multiple of
    /// `GATE_SPACING`, so the cells are the same either way.
    EnterHall { x: i64, y: i64, origin: i64 },
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
    /// The World Gates the character has gone through, either way, as the Hall
    /// gates the pairs are (`hall::GatePlace::number`): after `Welcome`, and
    /// whenever another is added. Their water is drawn orange.
    VisitedGates(Vec<u32>),
    /// Something has come into sight: a mob or another player. The map changing
    /// (`EnterLevel`, `EnterHall`, `OnWorldMap`, `Teleported`) forgets everything seen before.
    EntitySeen(EntityLook),
    /// Something in sight has moved.
    EntityMoved { id: Id, x: i64, y: i64 },
    /// Something has gone out of sight, or away.
    EntityGone { id: Id },
}
