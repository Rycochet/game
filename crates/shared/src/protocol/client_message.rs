//! What a client sends the server.

use serde::{Deserialize, Serialize};

use super::OnlineSettings;
use super::dev_mob::DevMobTool;
use super::dev_skills::DevSkills;
use super::dev_vitals::DevVitals;
use super::kind_change::KindChange;
use super::login::{CharacterId, Credentials, NewCharacter};
use crate::ability::{Ability, BarChange};
use crate::creature::{CreatureKind, Size};
use crate::difficulty::{Difficulties, Difficulty};
use crate::dungeon::Cell;
use crate::id::Id;
use crate::item::{Favour, Holder, ItemKind, Quality, Rotation};
use crate::item_form::RollRequest;
use crate::movement::Step;
use crate::poi::{ChunkPos, PoiKind};
use crate::site::DevArea;
use crate::world::ClockRate;

/// From a client to the server.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum ClientMessage {
    /// The first message on a new connection, answered with `LoginOptions`
    /// (the embedded server's own player, who needs no login: with
    /// `Characters`). `makes_characters`: the client has screens for choosing
    /// and making characters; without them the account's latest is played, or
    /// a plain one made. `difficulty`: the player's default, for a character
    /// with none saved (`Difficulty::for_existing`).
    Hello {
        protocol: u32,
        makes_characters: bool,
        difficulty: Difficulty,
    },
    /// Logs in, once `LoginOptions` has arrived, answered with `Characters`.
    Login(Credentials),
    /// Plays this character of the account's, answered with `Welcome`.
    ChooseCharacter { id: CharacterId },
    /// Deletes this character of the account's, save and all, while choosing:
    /// answered with `Characters` again.
    DeleteCharacter { id: CharacterId },
    /// Makes a new character and plays them, answered with `Welcome`. The
    /// server checks the name and how many the account may have (a guest one).
    CreateCharacter(NewCharacter),
    /// Changes the character's difficulty, answered with `Difficulty` either
    /// way: refused unless the server allows it, and never to or from Ironman.
    SetDifficulty(Difficulty),
    /// The difficulties the server allows (none counts as all of them): only
    /// the embedded server's own player, and not while `HOW_DIFFICULTIES` sets
    /// them. Everyone on one no longer allowed is moved to the nearest.
    SetDifficulties(Difficulties),
    /// Asks the server's clock, now and then. `sent` is the client's own time,
    /// handed back in `Pong` so it can tell how long the round trip took.
    Ping { sent: f64 },
    /// The player takes a step on the map they are on: the server checks
    /// it, moves them and sends what comes into sight, then (once a tick)
    /// `Position`. `map` counts the maps the client has been put on
    /// (`EnterLevel`, `OnWorldMap`) since `Welcome`, so a step meant for the
    /// last map is ignored on the next; `number` counts the client's steps.
    Step { map: u32, number: u32, step: Step },
    /// Go into the poi in this world map chunk (the player must be near
    /// enough, `poi::can_enter_from`): into the wilderness where it is set
    /// in, at its entrance or where they last walked out of it.
    EnterSite { chunk: ChunkPos },
    /// Use what the player stands on: stairs, or outside ground at the edge
    /// of the level (back to the world map).
    UseExit,
    /// Down the stairs underfoot if they go down; on the world map, explore
    /// the wilderness there (`EnterLevel`; refused in water).
    Down,
    /// Up the stairs underfoot if they go up; otherwise to the world map
    /// (`OnWorldMap`): the wilderness from outside ground, not into water; a
    /// site where `site::can_leave` allows; a Dev Area from anywhere.
    Up,
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
    /// Lays out a carried grid again, in one go (`Inventory::sort_grid`): what looks
    /// the same stacked, then largest first from the top left. Answered with
    /// `Inventory`; nothing changes if something wouldn't fit.
    SortGrid { holder: Holder },
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
    /// What these items are, for showing them where they are written into text
    /// (`crate::item_text`): answered with `ItemsLookedUp`. Only the first
    /// `MOST_LOOKED_UP` of a message are looked up.
    LookUpItems(Vec<Id>),
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
    /// Developer tools: new items of this kind in the character's inventory, or
    /// for `to`, made for this size (or the one getting them's own) and their
    /// fingers, answered with `Inventory`.
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
        /// What it is rolled from (`item_form::make_item`), so it is what Give Item
        /// showed; `None` for a random one.
        seed: Option<u64>,
        /// How many: kept to what the server allows (`StackLimits`).
        count: u32,
        /// All alike, in one stack; otherwise each rolled from its own seed
        /// (`item_form::batch_seed`). What doesn't fit goes on the ground in a site,
        /// and on the world map nothing is given unless all of it fits.
        identical: bool,
        /// What their form must be like (`item_form::RollRequest`); refused, giving
        /// nothing, if any of them can't keep to it (`try_make_item`).
        request: RollRequest,
        /// Who gets them: a mob or another player in sight (the target), or
        /// `None` for the character. Nothing is given if `to` isn't in sight.
        to: Option<Id>,
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
    /// Developer tools: the character becomes this kind of creature, for good or
    /// only its body (`how`). What its new body can't wear drops. Answered with
    /// `Sheet` and `Inventory`.
    DevSetCreature { kind: CreatureKind, how: KindChange },
    /// Developer tools: the character is deleted, save and all, answered with
    /// `Characters` to choose or make another.
    DevDeleteCharacter,
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
    /// Developer tools: to the Hall of Worlds, in front of the gate that leads
    /// to the World Gate nearest the player (or the Hall's first gate, on a
    /// planet that has none there), answered with `EnterHall`. Only an embedded
    /// server accepts it.
    DevToHall,
    /// Developer tools: lightning near the player now, whatever the weather:
    /// a flash for them and a bolt close by, which others in sight see too.
    DevLightning,
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
    /// Developer tools: move the world's time to this many game minutes since
    /// the world began, and/or have it run at another rate from now on, answered
    /// with `WorldClock` to everyone in that world.
    DevSetWorldClock {
        minutes: Option<f64>,
        rate: Option<ClockRate>,
    },
    /// Go through the World Gate `entrance` of this world (the player must be
    /// within `ENTER_REACH` of it, on the world map): into the Hall of Worlds,
    /// before the gate it leads to (`EnterHall`). Waits behind the steps there.
    EnterGate { entrance: u8 },
    /// Developer tools: to the world of this seed (any number; none: this one),
    /// at World Gate `gate` (from 0) if it has it, else where the player was,
    /// or a gate if new there; answered with `Welcome` (if new) and `OnWorldMap`.
    DevGoToWorld { seed: Option<u64>, gate: Option<u8> },
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
    /// Developer tools: make a Dev Area as if new and go into it; its way out
    /// leads back to where the player was before the first of them.
    DevGenerateArea(DevArea),
    /// Developer tools for mobs (`DevMobTool`): make, inspect, delete, freeze.
    DevMobs(DevMobTool),
    /// Developer tools: harm, spend, fill or raise the character's Health and
    /// Mana (`DevVitals`), answered with `Vitals`.
    DevVitals(DevVitals),
    /// Developer tools: practise some skills at random, or forget all but the
    /// kind's own (`DevSkills`), answered with `Sheet`.
    DevSkills(DevSkills),
    /// Change the action bar (`ability::BarChange`), answered with `ActionBar`:
    /// what it is now, whether the change was made or refused.
    ActionBar(BarChange),
    /// Use an ability on a creature, or on yourself when `target` is `None`
    /// (Heal), when there is the energy: answered with `AbilityStarted`, or
    /// nothing if it can't be done.
    UseAbility {
        ability: Ability,
        target: Option<Id>,
    },
    /// Call off the ability winding up, if it hasn't landed: answered with
    /// `AbilityLanded` and `Landed::Cancelled`.
    CancelAbility,
    /// Take back an ability asked for that is still waiting for the energy to
    /// begin (the player moved or chose another): one already begun is left
    /// alone, so nothing is lost for it. Not answered.
    WithdrawAbility,
}
