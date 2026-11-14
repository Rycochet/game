//! What a client sends the server.

use serde::{Deserialize, Serialize};

use super::login::{CharacterId, Credentials, NewCharacter};
use super::{
    DevAlignment, DevGive, DevMobTool, DevSkills, DevVitals, DoorAsk, KindChange, OnlineSettings,
};
use crate::ability::{Ability, BarChange};
use crate::creature::CreatureKind;
use crate::difficulty::{Difficulties, Difficulty};
use crate::dungeon::Cell;
use crate::id::Id;
use crate::item::{Holder, Rotation};
use crate::movement::Step;
use crate::poi::{ChunkPos, PoiKind};
use crate::site::DevArea;
use crate::spot::Spot;
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
    /// About roles: what the character is shown as, and the Accounts panel's.
    Role(super::RoleAsk),
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
    /// A step on the world map: checked, taken, then (once a tick)
    /// `Position`. `map` counts the maps since `Welcome` (a step meant for the
    /// last is ignored); `number` counts the client's steps and moves.
    Step { map: u32, number: u32, step: Step },
    /// A move on a level or in the Hall, straight to `to` (`protocol::moves`).
    Move { map: u32, number: u32, to: Spot },
    /// Go into the poi in this world map chunk (the player must be near
    /// enough, `poi::can_enter_from`): into the wilderness where it is set
    /// in, at its entrance or where they last walked out of it.
    EnterSite { chunk: ChunkPos },
    /// Use what is underfoot: stairs, or outside ground at the level's edge (to the world map).
    UseExit,
    /// Down the stairs underfoot if they go down; on the world map, explore
    /// the wilderness there (`EnterLevel`; refused in water).
    Down,
    /// Up the stairs underfoot if they go up; otherwise to the world map
    /// (`OnWorldMap`): the wilderness from outside ground, not into water; a
    /// site where `site::can_leave` allows; a Dev Area from anywhere.
    Up,
    /// Stop or restart the world's clock, answered with `Paused`. Only a server
    /// with just this player on it pauses (`Welcome`'s `can_pause`).
    SetPaused(bool),
    /// Opens the server to other players with these settings (or closes it, with
    /// `None`), answered with `Online`. Only the embedded server's own player
    /// may; others are ignored.
    SetOnline(Option<OnlineSettings>),
    /// Locks a networked server down (only accounts with Server Options may
    /// play; the others are sent away) or opens it again, answered with
    /// `Lockdown` to everyone who may change it. Only for accounts that may.
    SetLockdown(bool),
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
    /// The top `count` of a carried stack lifted out on its own: `then` is what to
    /// do with it, as for `FromStack`. For a counted stack (coins), whose part is
    /// new: the server gives it an id and puts that in `then` in place of the one
    /// it names (the client's stand-in). Answered as `then` is.
    PartOfStack {
        stack: Id,
        count: u32,
        then: Box<ClientMessage>,
    },
    /// Throws something the character carries in their bin, with all in it,
    /// destroying what was there, answered with `Inventory`.
    BinItem { item: Id },
    /// Gives something carried to a mob near enough and in sight, which remembers it,
    /// answered with `Inventory`, whether or not it could be done.
    GiveItem { item: Id, to: Id },
    /// Asks to trade with someone: for now a shop's keeper (`Keepers`), from beside
    /// its door, after any steps to get there. Answered with `TradeStock` if it can.
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
    /// Puts something carried or worn on the ground, in the nearest bag in the
    /// player's reach or a new one at their feet (`ground::nearest_in_reach`), in
    /// a site, after any moves to get there. Answered with `Inventory` and
    /// `GroundItems`, whether or not it could be done, as are the three below.
    DropItem { item: Id },
    /// Puts as much of something in a bag in the player's reach as fits on a
    /// carried stack that looks the same, the rest staying on the ground.
    StackFromGround { item: Id, onto: Id },
    /// Takes something from a bag in the player's reach, to this place.
    TakeFromGround {
        item: Id,
        to: Holder,
        x: u8,
        y: u8,
        rotation: Rotation,
    },
    /// Puts on something from a bag in the player's reach.
    WearFromGround { item: Id },
    /// Throws away something from a bag in the player's reach, as `BinItem`.
    BinFromGround { item: Id },
    /// Eats the top one of something carried or in a bag in the player's reach
    /// (`bonus::is_food`), in turn with steps: its effect starts (`Sheet::eat`).
    /// Answered with `Inventory` (and `GroundItems`), `Sheet` and `Vitals`.
    Consume { item: Id },
    /// Developer tools: new items of this kind for the character or `to`, made for this size
    /// (or the one getting them's own) and their fingers, answered with `Inventory`.
    DevGiveItem(DevGive),
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
    /// Developer tools: the character becomes this kind of creature, for good or only its
    /// body (`how`). What its new body can't wear drops. Answered with `Sheet` and `Inventory`.
    DevSetCreature { kind: CreatureKind, how: KindChange },
    /// Developer tools: the character is deleted, save and all, answered with
    /// `Characters` to choose or make another.
    DevDeleteCharacter,
    /// Developer tools: jump to a point on this map, in its measure
    /// (`protocol::moves`), answered with `Teleported`.
    DevTeleport { x: i64, y: i64 },
    /// Developer tools: back to where new characters start on the world map,
    /// from anywhere (out of a site too), answered with `OnWorldMap`. Only
    /// an embedded server accepts it; others ignore it.
    DevBackToStart,
    /// Developer tools: to the Hall of Worlds, before the gate leading to World
    /// Gate `gate` (from 0; none: the nearest, or another world's first) of the
    /// world with this `seed` (none: the player's own). Answered with `EnterHall`.
    DevToHall { seed: Option<u64>, gate: Option<u8> },
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
    /// Developer tools: show every poi, known or not (embedded server only).
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
    /// Developer tools: ask after or change the character's hidden alignment
    /// (`DevAlignment`), answered with `ServerMessage::DevAlignment`.
    DevAlignment(DevAlignment),
    /// Developer tools: change the character's skills, or `who`'s (another player
    /// in sight), answered with `DevSkillsOf` and their `Sheet`.
    DevSkills { who: Option<Id>, change: DevSkills },
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
    /// Open or close doors (`DoorAsk`), in turn with the player's steps.
    Door(DoorAsk),
    /// The player turned in place to face this way (free and at once); not answered.
    Face(crate::facing::Heading),
    /// About quests (`QuestAsk`).
    Quest(super::QuestAsk),
}
