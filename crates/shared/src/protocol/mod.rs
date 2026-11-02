//! Messages between the client and the server, and the `Link` that carries
//! them. See `docs/context/shared/protocol.md`.
//!
//! Clients send intents (`ClientMessage`); the server answers with what the
//! player may know (`ServerMessage`). Every message is encoded with postcard,
//! also in single player, where the link is an in-process channel.

mod level;
mod link;
mod login;

pub use level::{
    ENTER_REACH, LEVEL_CHUNK_SIZE, LevelChunk, LevelInfo, RememberedLevel, SeenStairs, level_chunk,
};
pub use link::{ClientLink, Link, LinkError, ServerLink, Wire, local_pair, wired};
pub use login::{CharacterId, CharacterSummary, Credentials, GuestToken, LoginMethod};

use serde::{Deserialize, Serialize};

use crate::creature::{CreatureKind, Size};
use crate::dungeon::Cell;
use crate::ground::Load;
use crate::id::Id;
use crate::item::{Favour, Holder, Inventory, Item, ItemKind, Quality, Rotation};
use crate::mob::EntityLook;
use crate::movement::Step;
use crate::poi::{CHUNK_SIZE, ChunkPos, Poi, PoiKind};
use crate::rng::Seed;
use crate::sheet::{Appearance, Sheet};
use crate::site::PoiLook;
use crate::world::WorldId;

/// Bump whenever a message changes, so mismatched clients and servers refuse
/// each other instead of misreading messages.
pub const PROTOCOL_VERSION: u32 = 33;

/// Where a server's WebSocket is, beside the web client's files: what
/// network clients connect to.
pub const SOCKET_PATH: &str = "/ws";

/// How far from the player (east–west and north–south, in grid points) the
/// client may show anything on the world map: the corners of its window
/// turned 45°, plus a token.
pub const WORLD_MAP_SIGHT: i64 = 300;

/// How much further than `WORLD_MAP_SIGHT` the server sends world map
/// chunks, so the client already has them when they come into sight.
pub const WORLD_MAP_PRELOAD: i64 = CHUNK_SIZE;

/// From a client to the server.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum ClientMessage {
    /// The first message on a new connection, answered with `LoginOptions`
    /// (an embedded server, which has no logins: with `Welcome`).
    Hello { protocol: u32 },
    /// Logs in, once `LoginOptions` has arrived. A guest is answered with
    /// `Welcome`, anyone else with `Characters`.
    Login(Credentials),
    /// Plays this character of the account's, answered with `Welcome`.
    ChooseCharacter { id: CharacterId },
    /// Makes a new character and plays them, answered with `Welcome`. The
    /// server checks the name and how many the account has.
    CreateCharacter {
        name: String,
        /// What they are: refused unless it is `CreatureDef::playable`.
        kind: CreatureKind,
        /// How they look: refused unless it suits the kind.
        appearance: Appearance,
    },
    /// Asks the server's clock, now and then. `sent` is the client's own time,
    /// handed back in `Pong` so it can tell how long the round trip took.
    Ping { sent: f64 },
    /// The player takes a step on the map they are on: the server checks
    /// it, moves them and sends what comes into sight, then (once a tick)
    /// `Position`. `map` counts the maps the client has been put on
    /// (`EnterLevel`, `OnWorldMap`) since `Welcome`, so a step meant for the
    /// last map is ignored on the next; `number` counts the client's steps.
    Step { map: u32, number: u32, step: Step },
    /// Go into the site of the poi in this world map chunk (the player
    /// must be at it).
    EnterSite { chunk: ChunkPos },
    /// Use what the player stands on: stairs, or outside ground at the edge
    /// of the level (back to the world map).
    UseExit,
    /// Stop or restart the world's clock, answered with `Paused`. Only a server
    /// with just this player on it pauses (`Welcome`'s `can_pause`); others
    /// ignore it.
    SetPaused(bool),
    /// Moves something the character carries to this place in this grid, turned
    /// this way, answered with `Inventory`, whether or not it could go.
    MoveItem {
        item: Id,
        to: Holder,
        x: u8,
        y: u8,
        rotation: Rotation,
    },
    /// Throws something the character carries in their bin, with all in it,
    /// destroying what was there, answered with `Inventory`.
    BinItem { item: Id },
    /// Gives something carried to a mob near enough and in sight, which remembers it,
    /// answered with `Inventory`, whether or not it could be done.
    GiveItem { item: Id, to: Id },
    /// Asks to trade with someone: for now a shop's keeper (`Keepers`), from beside
    /// its door, after any steps to get there. Answered with `TradeStock` if it can
    /// be.
    OpenTrade { with: Id },
    /// Takes something from who they are trading with, to this place. Answered with
    /// `Inventory` and `TradeStock`, whether or not it could be done.
    TakeItem {
        with: Id,
        item: Id,
        to: Holder,
        x: u8,
        y: u8,
        rotation: Rotation,
    },
    /// Puts on something the character carries, answered with `Inventory`, whether
    /// or not it could go on. Taking it off is `MoveItem`.
    WearItem { item: Id },
    /// Puts something carried or worn on the ground, on the cell the player is on, in
    /// a site, after any steps to get there. Answered with `Inventory` and
    /// `GroundItems`, whether or not it could be done, as are the three below.
    DropItem { item: Id },
    /// Takes something from the ground under the player, to this place.
    TakeFromGround {
        item: Id,
        to: Holder,
        x: u8,
        y: u8,
        rotation: Rotation,
    },
    /// Puts on something from the ground under the player.
    WearFromGround { item: Id },
    /// Throws away something from the ground under the player, as `BinItem`.
    BinFromGround { item: Id },
    /// Developer tools: a new item of this kind in the character's pockets if
    /// there is room, made for this size (or their own) and their fingers,
    /// answered with `Inventory`.
    DevGiveItem {
        kind: ItemKind,
        size: Option<Size>,
        magical: bool,
        /// Ignored for kinds without quality.
        quality: Quality,
        /// Part of a set, if it has quality.
        set: bool,
        /// Blessed or cursed. It and `magical` are as secret as on anything else.
        favour: Option<Favour>,
    },
    /// Developer tools: be told of every mob on the level, seen or not, or not.
    DevDetectMonsters(bool),
    /// Developer tools: truesight on or off. The client is told what is secret about
    /// items, without it being found out. Answered with `Inventory`.
    DevTruesight(bool),
    /// Developer tools: god mode on or off. Mobs take no notice of the character,
    /// and it can walk through them.
    DevGodMode(bool),
    /// Developer tools: this mob dies, if it is in sight. What it had falls where it
    /// stood, and it doesn't come back.
    DevKillMob { mob: Id },
    /// Developer tools: the character becomes this kind of creature. Everything
    /// comes off into the new pockets, and what doesn't fit is lost. Answered with
    /// `Inventory`.
    DevSetCreature(CreatureKind),
    /// Developer tools: jump to a point on the map the player is on (in a
    /// site, only onto a walkable cell), answered with `Teleported`. Only an
    /// embedded (single-player) server accepts it; others ignore it.
    DevTeleport { x: i64, y: i64 },
    /// Developer tools: back to where new characters start on the world map,
    /// from anywhere (out of a site too), answered with `OnWorldMap`. Only
    /// an embedded server accepts it; others ignore it.
    DevBackToStart,
    /// Developer tools: to the plain with every kind of poi (`shared::showcase`),
    /// from anywhere, answered with `OnWorldMap`. Only an embedded server accepts
    /// it, and only while the showcase is on.
    DevToShowcase,
    /// Developer tools: go straight into a site of any kind and seed, at
    /// any level, a building ruined this much (0 to 5). Only an embedded
    /// (single-player) server accepts it.
    DevEnterSite {
        kind: PoiKind,
        seed: u64,
        level: u32,
        ruin: u8,
    },
    /// Developer tools: change a cell of the level the player is on (not the
    /// one they stand on). Only an embedded server accepts it.
    DevSetCell { x: i64, y: i64, cell: Cell },
    /// Developer tools: set the snow line (which freezes water), answered
    /// with `SnowLine`. Only an embedded server accepts it.
    DevSnowLine(u32),
    /// Developer tools: move the world's clock to this many game minutes since
    /// the world began, answered with `Clock` to everyone playing.
    DevSetTime { minutes: f64 },
    /// Developer tools: show every poi, known or not, for this connection.
    /// Only an embedded server accepts it.
    DevRevealPois,
    /// Developer tools: the character knows the whole of the level they are on,
    /// answered as if arriving there again. Not in towns, known already.
    DevRevealMap,
    /// Developer tools: the character forgets the level they are on, answered
    /// as if arriving there for the first time; or on the world map, all of it,
    /// answered with `ForgetWorldMap` and what is in sight. Not in towns.
    DevForgetMap,
    /// Developer tools: as `DevForgetMap`, and the server forgets it too: the
    /// site's save, or on the world map every site's, so they are made anew
    /// with their generators' current versions. Anyone else in a forgotten site
    /// is put out at its poi (`OnWorldMap`). Towns too.
    DevForgetMapOnServer,
    /// Developer tools: make a site of this kind, seed and ruin (0 to 5) as if
    /// new, and go straight into it, where the player stands on the world map.
    /// It is not added to the world map.
    DevGenerateSite { kind: PoiKind, seed: u64, ruin: u8 },
}

impl ClientMessage {
    /// Whether it is a developer tools message (`Dev...`), which only an
    /// embedded server accepts.
    pub fn is_dev_tool(&self) -> bool {
        matches!(
            self,
            Self::DevTeleport { .. }
                | Self::DevBackToStart
                | Self::DevToShowcase
                | Self::DevEnterSite { .. }
                | Self::DevSetCell { .. }
                | Self::DevSnowLine(_)
                | Self::DevSetTime { .. }
                | Self::DevRevealPois
                | Self::DevRevealMap
                | Self::DevForgetMap
                | Self::DevForgetMapOnServer
                | Self::DevGenerateSite { .. }
                | Self::DevGiveItem { .. }
                | Self::DevSetCreature(_)
                | Self::DevDetectMonsters(_)
                | Self::DevTruesight(_)
                | Self::DevGodMode(_)
                | Self::DevKillMob { .. }
        )
    }

    /// A `Ping` as sent, for whoever carries the bytes to time it exactly.
    pub fn ping_bytes(sent: f64) -> Vec<u8> {
        postcard::to_allocvec(&Self::Ping { sent }).expect("messages always encode")
    }

    /// The `sent` of a `Ping`, if these bytes are one. Only decodes messages
    /// of a `Ping`'s size, so it can be asked of every message.
    pub fn read_ping(bytes: &[u8]) -> Option<f64> {
        if bytes.len() != PING_BYTES {
            return None;
        }
        match postcard::from_bytes(bytes) {
            Ok(Self::Ping { sent }) => Some(sent),
            _ => None,
        }
    }
}

/// The size of a `Ping` as sent: its kind, then a float.
const PING_BYTES: usize = 9;

/// The size of a `Pong` as sent: its kind, then two floats.
const PONG_BYTES: usize = 17;

impl ServerMessage {
    /// A `Pong` as sent, for whoever carries the bytes to answer straight away.
    pub fn pong_bytes(sent: f64, clock: f64) -> Vec<u8> {
        postcard::to_allocvec(&Self::Pong { sent, clock }).expect("messages always encode")
    }

    /// The `sent` and `clock` of a `Pong`, if these bytes are one. Only decodes
    /// messages of a `Pong`'s size, so it can be asked of every message.
    pub fn read_pong(bytes: &[u8]) -> Option<(f64, f64)> {
        if bytes.len() != PONG_BYTES {
            return None;
        }
        match postcard::from_bytes(bytes) {
            Ok(Self::Pong { sent, clock }) => Some((sent, clock)),
            _ => None,
        }
    }
}

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

/// What a player knows of one world map chunk. The terrain itself isn't
/// sent: the client generates it from the terrain seed. Later this also
/// carries the chunk's changes and per-player state (visited, attitude).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct WorldChunk {
    pub chunk: ChunkPos,
    /// Its poi, if it has one.
    pub poi: Option<SeenPoi>,
}

/// A world map poi as a player knows it: what and where it is, not
/// the seed its site comes from.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct SeenPoi {
    pub kind: PoiKind,
    pub look: PoiLook,
    pub chunk: ChunkPos,
    pub x: i64,
    pub y: i64,
}

impl From<Poi> for SeenPoi {
    fn from(poi: Poi) -> Self {
        Self {
            kind: poi.kind,
            look: PoiLook::of(poi.kind, poi.seed),
            chunk: poi.chunk,
            x: poi.x,
            y: poi.y,
        }
    }
}

/// What a client is told about a world when it joins.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct WorldInfo {
    pub id: WorldId,
    /// Public: the client generates the world map terrain from it.
    pub terrain_seed: Seed,
    /// Which world map generator the world was made with (`WORLD_MAP_VERSION`).
    pub world_map_version: u32,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::site::{StairsDirection, StairsStyle};

    #[test]
    fn pings_and_pongs_are_recognised_from_their_bytes() {
        let ping = ClientMessage::ping_bytes(1.5);
        assert_eq!(ClientMessage::read_ping(&ping), Some(1.5));
        let pong = ServerMessage::pong_bytes(1.5, 6.75);
        assert_eq!(ServerMessage::read_pong(&pong), Some((1.5, 6.75)));

        // Whatever else is sent, however long, isn't taken for one.
        let (client, server) = every_message();
        for message in client {
            let bytes = postcard::to_allocvec(&message).unwrap();
            assert_eq!(ClientMessage::read_ping(&bytes), None, "{message:?}");
        }
        for message in server {
            let bytes = postcard::to_allocvec(&message).unwrap();
            let is_pong = matches!(message, ServerMessage::Pong { .. });
            assert_eq!(ServerMessage::read_pong(&bytes).is_some(), is_pong);
        }
    }

    /// One of every message, so a message that can't be sent fails here.
    fn every_message() -> (Vec<ClientMessage>, Vec<ServerMessage>) {
        let client = vec![
            ClientMessage::Hello {
                protocol: PROTOCOL_VERSION,
            },
            ClientMessage::Login(Credentials::Guest(GuestToken([7; 32]))),
            ClientMessage::ChooseCharacter {
                id: CharacterId(crate::id::Id::for_test(1)),
            },
            ClientMessage::CreateCharacter {
                name: "Some One".into(),
                kind: CreatureKind::Kobold,
                appearance: Appearance::default(),
            },
            ClientMessage::Step {
                map: 2,
                number: 9,
                step: Step::new(-1, 1).unwrap(),
            },
            ClientMessage::EnterSite {
                chunk: ChunkPos { x: 2, y: -1 },
            },
            ClientMessage::UseExit,
            ClientMessage::SetPaused(true),
            ClientMessage::MoveItem {
                item: Id(uuid::Uuid::from_u128(7)),
                to: Holder::Inside(Id(uuid::Uuid::from_u128(8))),
                x: 2,
                y: 3,
                rotation: Rotation::Quarter,
            },
            ClientMessage::BinItem {
                item: Id(uuid::Uuid::from_u128(7)),
            },
            ClientMessage::GiveItem {
                item: Id(uuid::Uuid::from_u128(7)),
                to: Id(uuid::Uuid::from_u128(8)),
            },
            ClientMessage::WearItem {
                item: Id(uuid::Uuid::from_u128(7)),
            },
            ClientMessage::DevGiveItem {
                kind: ItemKind::Pouch,
                size: Some(Size::Large),
                magical: true,
                quality: Quality::Epic,
                set: false,
                favour: Some(Favour::Cursed),
            },
            ClientMessage::OpenTrade {
                with: Id::for_test(3),
            },
            ClientMessage::TakeItem {
                with: Id::for_test(3),
                item: Id::for_test(4),
                to: Holder::Pockets,
                x: 1,
                y: 2,
                rotation: Rotation::Quarter,
            },
            ClientMessage::DropItem {
                item: Id::for_test(5),
            },
            ClientMessage::TakeFromGround {
                item: Id::for_test(6),
                to: Holder::Pockets,
                x: 1,
                y: 2,
                rotation: Rotation::Quarter,
            },
            ClientMessage::WearFromGround {
                item: Id::for_test(7),
            },
            ClientMessage::BinFromGround {
                item: Id::for_test(8),
            },
            ClientMessage::DevSetCreature(CreatureKind::Chicken),
            ClientMessage::DevDetectMonsters(true),
            ClientMessage::DevTruesight(true),
            ClientMessage::DevGodMode(true),
            ClientMessage::DevKillMob {
                mob: Id::for_test(9),
            },
            ClientMessage::DevTeleport { x: -3, y: 4 },
            ClientMessage::DevSnowLine(40),
            ClientMessage::DevSetTime { minutes: 90.5 },
            ClientMessage::DevBackToStart,
            ClientMessage::DevToShowcase,
            ClientMessage::DevRevealPois,
            ClientMessage::DevRevealMap,
            ClientMessage::DevForgetMap,
            ClientMessage::DevForgetMapOnServer,
            ClientMessage::DevGenerateSite {
                kind: PoiKind::Castle,
                seed: 9,
                ruin: 2,
            },
            ClientMessage::DevEnterSite {
                kind: PoiKind::Crypt,
                seed: 3,
                level: 1,
                ruin: 2,
            },
        ];
        let server = vec![
            ServerMessage::LoginOptions {
                methods: vec![LoginMethod::Guest],
            },
            ServerMessage::Characters {
                list: vec![CharacterSummary {
                    id: CharacterId(crate::id::Id::for_test(1)),
                    name: "Some One".into(),
                    last_played: 99,
                }],
                can_create: true,
            },
            ServerMessage::Welcome {
                world: WorldInfo {
                    id: crate::world::WorldSeed::new(7).id(),
                    terrain_seed: Seed::new(8),
                    world_map_version: 1,
                },
                clock: 5.25,
                snow_line: 128,
                x: -4,
                y: 9,
                dev_tools: true,
                can_pause: true,
            },
            ServerMessage::ForgetWorldMap,
            ServerMessage::SnowLine(40),
            ServerMessage::Hour(6),
            ServerMessage::Clock(405.0),
            ServerMessage::Paused {
                paused: true,
                clock: 405.0,
            },
            ServerMessage::Pong {
                sent: 1.5,
                clock: 6.75,
            },
            ServerMessage::Refused("no".into()),
            ServerMessage::TradeStock {
                with: Id::for_test(3),
                items: Vec::new(),
            },
            ServerMessage::GroundItems {
                x: 1,
                y: -2,
                items: Vec::new(),
            },
            ServerMessage::GroundSeen {
                x: 1,
                y: -2,
                load: Load::Half,
            },
            ServerMessage::GroundGone { x: 1, y: -2 },
            ServerMessage::Keepers(vec![(3, Id::for_test(4))]),
            ServerMessage::WorldChunk(WorldChunk {
                chunk: ChunkPos { x: 1, y: -2 },
                poi: Some(SeenPoi {
                    kind: PoiKind::Castle,
                    look: PoiLook::Town { buildings: 9 },
                    chunk: ChunkPos { x: 1, y: -2 },
                    x: 70,
                    y: -100,
                }),
            }),
            ServerMessage::WorldChunk(WorldChunk {
                chunk: ChunkPos { x: 0, y: 0 },
                poi: None,
            }),
            ServerMessage::EnterLevel(LevelInfo {
                kind: PoiKind::RockCave,
                ruined: true,
                building: false,
                index: 1,
                levels: 3,
                basement: false,
                bounds: (-20, -10, 40, 30),
                arrival: (4, 5),
                entering_site: true,
            }),
            ServerMessage::RememberedLevel(RememberedLevel {
                index: 2,
                bounds: (-20, -10, 40, 30),
                chunks: vec![LevelChunk {
                    chunk: (0, 0),
                    mask: 1,
                    cells: vec![Cell::Floor],
                    stairs: Vec::new(),
                    remembered: true,
                }],
            }),
            ServerMessage::Buildings(vec![crate::site::BuildingLook {
                id: 3,
                kind: crate::site::BuildingKind::Bank,
                walls: crate::site::WallKind::Brick,
                roof: crate::site::RoofKind::SlateTiles,
                rects: vec![crate::site::Rect {
                    x: 1,
                    y: -2,
                    width: 3,
                    height: 4,
                }],
                door: (2, -2),
                faces: (0, -1),
            }]),
            ServerMessage::Fields(vec![crate::site::FieldLook {
                rect: crate::site::Rect {
                    x: -12,
                    y: 3,
                    width: 8,
                    height: 9,
                },
                livestock: true,
                variety: 77,
            }]),
            ServerMessage::LevelCells(LevelChunk {
                chunk: (-1, 2),
                mask: 0b101,
                cells: vec![Cell::Floor, Cell::Door],
                stairs: vec![SeenStairs {
                    open_to_sky: false,
                    x: -6,
                    y: 16,
                    direction: StairsDirection::Down,
                    descends: (0, 1),
                    style: StairsStyle::Spiral,
                }],
                remembered: true,
            }),
            ServerMessage::OnWorldMap { x: 100, y: -4 },
            ServerMessage::Teleported { x: 6, y: -7 },
            ServerMessage::Position {
                map: 1,
                step: 30,
                x: -2,
                y: 5,
            },
            ServerMessage::Sheet(Sheet::new(CreatureKind::Horse)),
            ServerMessage::EntitySeen(EntityLook {
                id: Id(uuid::Uuid::from_u128(4)),
                kind: CreatureKind::Giant,
                appearance: Appearance::default(),
                feeling: crate::sheet::Feeling::Friendly,
                x: 3,
                y: -4,
            }),
            ServerMessage::EntityMoved {
                id: Id(uuid::Uuid::from_u128(4)),
                x: 4,
                y: -4,
            },
            ServerMessage::EntityGone {
                id: Id(uuid::Uuid::from_u128(4)),
            },
            ServerMessage::Inventory({
                let mut inventory = Inventory::default();
                let pouch = crate::item::Item::new(Id(uuid::Uuid::from_u128(1)), ItemKind::Pouch);
                inventory.give(pouch).unwrap();
                inventory
            }),
        ];
        (client, server)
    }

    #[test]
    fn every_message_survives_the_link() {
        let (client, server) = local_pair();
        let (to_server, to_client) = every_message();
        for message in &to_server {
            assert!(client.send(message));
            assert_eq!(server.receive(), Ok(Some(message.clone())));
        }
        for message in &to_client {
            assert!(server.send(message));
            assert_eq!(client.receive(), Ok(Some(message.clone())));
        }
        assert_eq!(client.receive(), Ok(None));
    }

    #[test]
    fn a_link_knows_when_the_other_end_has_gone() {
        let (client, server) = local_pair();
        drop(server);
        assert_eq!(client.receive(), Err(LinkError::Closed));
        assert!(!client.send(&ClientMessage::UseExit));
    }

    #[test]
    fn wires_carry_the_bytes_both_ways() {
        // Two wired links joined by their wires, as over a socket.
        let (client, client_wire) = wired::<ClientMessage, ServerMessage>();
        let (server, server_wire) = wired::<ServerMessage, ClientMessage>();
        let (to_server, to_client) = every_message();
        for message in &to_server {
            assert!(client.send(message));
            let bytes = client_wire.take().unwrap().unwrap();
            assert!(server_wire.deliver(bytes));
            assert_eq!(server.receive(), Ok(Some(message.clone())));
        }
        for message in &to_client {
            assert!(server.send(message));
            let bytes = server_wire.take().unwrap().unwrap();
            assert!(client_wire.deliver(bytes));
            assert_eq!(client.receive(), Ok(Some(message.clone())));
        }
        // Garbage from the socket is noticed, not misread.
        assert!(server_wire.deliver(vec![0xff, 0xff, 0xff]));
        assert_eq!(server.receive(), Err(LinkError::Garbled));
        // Either side going is noticed by the other.
        drop(client);
        assert_eq!(client_wire.take(), Err(LinkError::Closed));
        drop(server_wire);
        assert_eq!(server.receive(), Err(LinkError::Closed));
    }
}
