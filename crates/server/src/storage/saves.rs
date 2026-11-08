//! What is saved: worlds, sites, levels, mobs, buildings, characters and visits.

use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use shared::ability::ActionBar;
use shared::dungeon::Cell;
use shared::ground::Load;
use shared::id::Id;
use shared::item::{Inventory, Item};
use shared::mob::Memory;
use shared::poi::{ChunkPos, PoiKind};
use shared::sheet::Sheet;
use shared::site::DevArea;
use shared::world::{ClockRate, WorldId, WorldSeed};
use std::collections::{BTreeMap, BTreeSet};

/// Which site: a world map poi's, or a made-up one (dev tools).
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum SiteId {
    /// The site of the poi in this world map chunk.
    Poi(ChunkPos),
    /// Developer tools: a site of this kind and map seed, a building ruined
    /// this much.
    Dev { kind: PoiKind, seed: u64, ruin: u8 },
    /// Developer tools: a Dev Area made from these choices. Never kept: it is
    /// forgotten when made again and when left.
    DevArea(DevArea),
    /// The world's wilderness (`shared::wilderness`): what players have
    /// changed of it, and what they have left there.
    Wilderness,
}

/// `world.ron`.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct WorldSave {
    /// The `SAVE_FORMAT` it was written with.
    pub format: u32,
    /// Secret: never sent to a client.
    pub seed: WorldSeed,
    /// The world map generator it was made with (`WORLD_MAP_VERSION`).
    pub world_map_version: u32,
    /// Its time when saved: game minutes since it began.
    pub clock: f64,
    /// The server's clock (`ServerSave::clock`) when `clock` was saved: the
    /// world's time has run on since, at `rate`. `None` from before worlds went
    /// by the server's clock, when it stopped while the server was off.
    #[serde(default)]
    pub server_clock: Option<f64>,
    /// How fast its time runs against the server's clock.
    #[serde(default)]
    pub rate: ClockRate,
    /// The action clock when saved, from before the server had one of its own
    /// (`ServerSave::action_ticks`): only read if that isn't saved yet.
    #[serde(default)]
    pub action_ticks: u64,
}

/// `server.ron`: the server's own clocks, which every world goes by.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct ServerSave {
    /// The `SAVE_FORMAT` it was written with.
    pub format: u32,
    /// Real seconds the server has run, not counting pauses (`clock::ServerClock`).
    pub clock: f64,
    /// Ticks of action time (`clock::ActionClock`).
    pub action_ticks: u64,
}

/// A site's file: made the first time anyone enters it.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct SiteSave {
    /// The `FORGET_BEFORE` it was saved under.
    pub forgotten_to: u32,
    /// The generator version it was first made with (`Generator::version`):
    /// it is always generated with this one.
    pub version: u32,
    /// Its levels that have been visited, by index.
    pub levels: BTreeMap<u32, LevelSave>,
}

/// A visited site level.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct LevelSave {
    /// When someone was last on it (seconds since 1970): it resets after
    /// nobody has been for a while, once there are spawns to bring back.
    pub last_visit: u64,
    /// Cells changed by players (dug, broken), applied over the generated
    /// level every time; a reset never undoes them.
    pub changes: BTreeMap<(i64, i64), Cell>,
    /// A town's buildings that players have dealt with, by `RoomId`.
    pub buildings: BTreeMap<u16, BuildingSave>,
    /// The ids of the level's mobs, by which of its spawns they are. A mob keeps its
    /// id while it lives, so what it remembers stays with it.
    pub mobs: BTreeMap<u32, Id>,
    /// What mobs have come to know and own, kept while they live.
    pub records: BTreeMap<Id, MobRecord>,
    /// Mobs that weren't at rest when last saved, by number, to carry on.
    pub unsettled: BTreeMap<u32, SavedMob>,
    /// What is lying on the ground, by cell, oldest first. Stays until taken: a
    /// reset never touches it.
    #[serde(default)]
    pub ground: BTreeMap<(i64, i64), Vec<Item>>,
    /// Which of its spawns have died, by number: they aren't made again.
    #[serde(default)]
    pub dead: BTreeSet<u32>,
    /// Everyone's scent marks, while any nose could still smell them.
    #[serde(default)]
    pub scents: crate::entities::Scents,
    /// The wilderness's only: blocks whose mobs have something saved (`dead`,
    /// `records`), by `(x, y)`, with when someone was last near (seconds
    /// since 1970): they reset after nobody has been for a while.
    #[serde(default)]
    pub wild_blocks: BTreeMap<(i64, i64), u64>,
}

/// What a mob has come to know and own.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct MobRecord {
    pub memory: Memory,
    pub carried: Vec<Item>,
    /// What it wears and carries: rolled when it is first needed, then its own.
    #[serde(default)]
    pub gear: Option<Inventory>,
}

/// What a mob was doing, for carrying on after the server has been off.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct SavedMob {
    pub position: (i64, i64),
    pub doing: crate::entities::Doing,
}

/// What players have done with a town's building.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct BuildingSave {
    /// Trades made by players, on top of the number it was generated with.
    pub trades: u32,
    /// The server's clock (game minutes) when last interacted with, which will
    /// decide when its stock refreshes. The clock can be moved back (dev tools),
    /// so this can be in the future: count that as no time having passed.
    pub last_interaction: Option<f64>,
    /// Who keeps it, if it is a shop: a mob never on the map, in the level's
    /// `records`, whose `carried` is what the shop has.
    #[serde(default)]
    pub keeper: Option<Id>,
    /// What it had for sale before it had a keeper: moved to them when they come.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub stock: Vec<Item>,
    /// The game minute it last finished making something.
    #[serde(default)]
    pub last_made: Option<u64>,
}

/// Cells of a level by chunk: one bit per cell (see
/// `shared::protocol::level_chunk`).
pub type CellMasks = BTreeMap<(i64, i64), u64>;

/// A character's file.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct CharacterSave {
    /// The world they are in.
    pub world: WorldId,
    /// What they know of each world they have been to, kept for going back.
    pub worlds: BTreeMap<WorldId, CharacterWorldSave>,
    /// What they carry, wherever they go.
    pub inventory: Inventory,
    /// What they are like, wherever they go.
    pub sheet: Sheet,
    /// The World Gates they have gone through, either way, by the Hall gate
    /// each pair is (`hall::GatePlace::number`): gates join worlds, so this is
    /// kept for them all.
    #[serde(default)]
    pub gates: BTreeSet<u32>,
    /// How long they have been played, in action ticks: only while their
    /// player was doing things (`connections::playtime`). Missing from older
    /// saves.
    #[serde(default)]
    pub playtime: u64,
    /// Their action bar. Older saves start with the new character's (Attack first).
    #[serde(default)]
    pub action_bar: ActionBar,
    /// How hard the game is for them; missing from older saves, which take
    /// their player's default on joining.
    #[serde(default)]
    pub difficulty: Option<shared::difficulty::Difficulty>,
}

/// What a character knows of one world, and where they are in it.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct CharacterWorldSave {
    /// The `FORGET_BEFORE` it was saved under.
    pub forgotten_to: u32,
    /// Where they are on the world map (at the poi, while in a site).
    pub world_position: (i64, i64),
    /// The site level they are on, if any.
    pub visit: Option<VisitSave>,
    /// Where they are in the Hall of Worlds, if they are there, having come
    /// from this world.
    #[serde(default)]
    pub hall: Option<(i64, i64)>,
    /// The world map chunks whose poi they have seen, and what they
    /// know of it.
    pub pois: BTreeMap<ChunkPos, KnownPoi>,
    /// The cells they have seen of each site level.
    pub memory: BTreeMap<SiteId, BTreeMap<u32, CellMasks>>,
    /// The bags they last saw on each site level, by cell, as they looked then.
    #[serde(default)]
    pub bags: BTreeMap<SiteId, BTreeMap<u32, RememberedBags>>,
    /// The generator version of each site as they last knew it: when a site's
    /// has changed, so has its map, and what they remember of it means nothing.
    pub site_versions: BTreeMap<SiteId, u32>,
    /// The level and cell they last walked out of each site from: going in
    /// again puts them back there.
    #[serde(default)]
    pub left_at: BTreeMap<SiteId, (u32, (i64, i64))>,
    /// Developer tools: where leaving a Dev Area takes them, while in one.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub dev_return: Option<DevReturn>,
}

/// Where a character was before going into a Dev Area, to go back to.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct DevReturn {
    pub world_position: (i64, i64),
    /// The site level, or the wilderness, they were on, if any.
    pub visit: Option<VisitSave>,
    /// Where in the Hall they were, if there.
    pub hall: Option<(i64, i64)>,
}

impl CharacterWorldSave {
    /// Forgets everything they knew of a site: what they saw, the bags there,
    /// its version and where they walked out.
    pub fn forget_site(&mut self, id: &SiteId) {
        self.memory.remove(id);
        self.bags.remove(id);
        self.site_versions.remove(id);
        self.left_at.remove(id);
    }

    /// Forgets every site, as `forget_site` does.
    pub fn forget_sites(&mut self) {
        self.memory.clear();
        self.bags.clear();
        self.site_versions.clear();
        self.left_at.clear();
    }
}

/// The bags a character remembers on a level, by cell.
pub type RememberedBags = BTreeMap<(i64, i64), Load>;

/// Where in a site a character is.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct VisitSave {
    pub site: SiteId,
    pub level: u32,
    pub position: (i64, i64),
    /// They walked into the site from the wilderness, where leaving it takes
    /// them back.
    #[serde(default)]
    pub from_wilderness: bool,
}

/// What a character knows of a poi they have seen.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct KnownPoi {
    /// Whether they have learned where the chunk's poi is. Towns are always
    /// known, and nothing teaches a character the others yet.
    pub known: bool,
    /// Whether they have been inside.
    pub visited: bool,
}
