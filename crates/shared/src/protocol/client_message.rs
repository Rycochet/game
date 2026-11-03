//! What a client sends the server.

use serde::{Deserialize, Serialize};

use super::OnlineSettings;
use super::login::{CharacterId, Credentials};
use crate::creature::{CreatureKind, Size};
use crate::dungeon::Cell;
use crate::id::Id;
use crate::item::{Favour, Holder, ItemKind, Quality, Rotation};
use crate::movement::Step;
use crate::poi::{ChunkPos, PoiKind};
use crate::sheet::Appearance;

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
    /// Opens the server to other players with these settings (or closes it, with
    /// `None`), answered with `Online`. Only the embedded server's own player
    /// may; others are ignored.
    SetOnline(Option<OnlineSettings>),
    /// Moves something the character carries to this place in this grid, turned
    /// this way, answered with `Inventory`, whether or not it could go.
    MoveItem {
        item: Id,
        to: Holder,
        x: u8,
        y: u8,
        rotation: Rotation,
    },
    /// Puts as much of a carried stack as fits on another carried stack that looks
    /// the same, from its top, the rest staying where it was. Answered with
    /// `Inventory`, whether or not it could be done.
    StackItem { item: Id, onto: Id },
    /// Some of the items of a carried stack (a mixed one), lifted out together, keeping
    /// their order: `then` is what to do with them, by the id of the lowest. It is
    /// one of `MoveItem`, `StackItem`, `GiveItem`, `WearItem`, `BinItem` or
    /// `DropItem`, and answered as that is.
    FromStack {
        stack: Id,
        items: Vec<Id>,
        then: Box<ClientMessage>,
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
    /// Puts as much of something on the ground under the player as fits on a
    /// carried stack that looks the same, the rest staying on the ground.
    StackFromGround { item: Id, onto: Id },
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
    /// items and sees secret doors, without any being found out. Answered with
    /// `Inventory`, and the level again if it has secret doors.
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
}
