//! The server's saved data: what is in each save, and loading and saving them.
//! Where they are kept is behind `saves::SaveBackend`, and writing them goes
//! through the save thread (`save_queue`).
//!
//! Everything is in the server's own data folder, never shared with a
//! client's (see `docs/context/architecture.md`), as RON files, readable by
//! people. Fields added later must have `#[serde(default)]`, so older saves
//! still load.

use std::collections::{BTreeMap, BTreeSet};
use std::env;
#[cfg(test)]
use std::path::Path;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use shared::dungeon::Cell;
use shared::ground::Load;
use shared::id::Id;
use shared::item::{Inventory, Item};
use shared::mob::Memory;
use shared::poi::{ChunkPos, PoiKind};
use shared::protocol::CharacterId;
use shared::sheet::Sheet;
use shared::site::FORGET_BEFORE;
use shared::world::{WorldId, WorldSeed};

use crate::save_queue::{FINISH_WITHIN, SaveQueue};
use crate::saves::{LoadError, Save, SaveKey};

/// Overrides where the server keeps its data (a folder path).
pub const DATA_FOLDER_VARIABLE: &str = "HOW_DATA";

/// Bump when a save changes in a way older servers can't read (adding a
/// field with a default doesn't need it). A server refuses a world saved
/// with a newer format than its own.
pub const SAVE_FORMAT: u32 = 3;

/// Saves from before this format are moved out of the way on starting
/// (`saves::set_aside_old_saves`): raising it wipes every server's saves.
pub const OLDEST_SAVE_FORMAT: u32 = 3;

/// The server's data folder: `HOW_DATA` if set, otherwise
/// `Hall of Worlds/server` in the user's data folder (`%APPDATA%` on
/// Windows), otherwise `server-data` here. A client keeps its own data
/// beside it, never in it.
pub fn default_data_folder() -> PathBuf {
    if let Some(folder) = env::var_os(DATA_FOLDER_VARIABLE) {
        return folder.into();
    }
    match dirs::data_dir() {
        Some(data) => data.join("Hall of Worlds").join("server"),
        None => "server-data".into(),
    }
}

/// One world's saves and the server's characters, or nowhere (tests: nothing is
/// saved). Saving only queues: it can't fail or hold up the game.
#[derive(Resource, Clone)]
pub struct Storage {
    queue: Option<SaveQueue>,
    world: WorldId,
    /// Saves from before this are ignored (`FORGET_BEFORE`).
    forget_before: u32,
}

impl Storage {
    /// Whether anything is saved at all.
    pub fn saves(&self) -> bool {
        self.queue.is_some()
    }

    /// Nothing is read or written.
    pub fn none() -> Self {
        Self {
            queue: None,
            world: WorldId(uuid::Uuid::nil()),
            forget_before: FORGET_BEFORE,
        }
    }

    /// The saves of world `id`, through `queue`.
    pub(crate) fn new(queue: SaveQueue, id: WorldId) -> Self {
        Self {
            queue: Some(queue),
            world: id,
            forget_before: FORGET_BEFORE,
        }
    }

    /// The files of world `id` in the data folder `data`.
    #[cfg(test)]
    pub fn in_folder(data: &Path, id: WorldId) -> Self {
        Self::new(SaveQueue::in_folder(data), id)
    }

    /// The saved world, if there is one. Anything wrong with it is an error, as
    /// it can't be started again: nothing is put aside.
    pub fn load_world(&self) -> Result<Option<WorldSave>, String> {
        let Some(queue) = &self.queue else {
            return Ok(None);
        };
        match queue.read(&SaveKey::World(self.world)) {
            Ok(Some(Save::World(save))) => Ok(Some(save)),
            Ok(_) => Ok(None),
            Err(LoadError::CantReach(error) | LoadError::Broken(error)) => Err(error),
        }
    }

    pub fn save_world(&self, world: &WorldSave) {
        if let Some(queue) = &self.queue {
            queue.save(SaveKey::World(self.world), Save::World(world.clone()));
        }
    }

    /// Waits for everything saved so far to be written (the server is stopping),
    /// within `FINISH_WITHIN`, logging what wasn't.
    pub(crate) fn finish_writing(&self) {
        if let Some(queue) = &self.queue
            && !queue.flush(FINISH_WITHIN)
        {
            error!("Not everything could be saved");
        }
    }

    /// Waits for everything saved so far to be written, within
    /// `FINISH_WITHIN`, for the debug API: `false` if some wasn't.
    #[cfg(feature = "debug-api")]
    pub(crate) fn flush(&self) -> bool {
        self.queue
            .as_ref()
            .is_none_or(|queue| queue.flush(FINISH_WITHIN))
    }

    /// Whether a save made under `forgotten_to` still counts.
    fn not_forgotten(&self, forgotten_to: u32, what: &str) -> bool {
        let kept = forgotten_to >= self.forget_before;
        if !kept {
            info!("Forgetting {what}: saved before {}", self.forget_before);
        }
        kept
    }

    /// A saved site, if it has been entered before. A broken one is put aside
    /// (and logged), so the site starts again; one from before `forget_before` is
    /// ignored, and saved over. An error if it couldn't be reached: it may be
    /// fine, so it mustn't be started again.
    pub fn load_site(&self, site: SiteId) -> Result<Option<SiteSave>, String> {
        let Some(queue) = &self.queue else {
            return Ok(None);
        };
        Ok(match queue.load(&SaveKey::Site(self.world, site))? {
            Some(Save::Site(save)) => self
                .not_forgotten(save.forgotten_to, &format!("{site:?}"))
                .then_some(save),
            _ => None,
        })
    }

    /// Deletes a site's save, if it has one.
    pub fn remove_site(&self, site: SiteId) {
        if let Some(queue) = &self.queue {
            queue.remove(SaveKey::Site(self.world, site));
        }
    }

    /// Deletes every site's save.
    pub fn remove_sites(&self) {
        if let Some(queue) = &self.queue {
            queue.remove(SaveKey::Sites(self.world));
        }
    }

    pub fn save_site(&self, site: SiteId, save: &SiteSave) {
        if let Some(queue) = &self.queue {
            let forgotten_to = self.forget_before;
            queue.save(
                SaveKey::Site(self.world, site),
                Save::Site(SiteSave {
                    forgotten_to,
                    ..save.clone()
                }),
            );
        }
    }

    /// A character's whole save, if they have one. A broken one is put aside (and
    /// logged), so they start again. An error if it couldn't be reached: it may
    /// be fine, and starting again would save a new character over it.
    fn character(&self, id: &CharacterId) -> Result<Option<CharacterSave>, String> {
        let Some(queue) = &self.queue else {
            return Ok(None);
        };
        Ok(match queue.load(&SaveKey::Character(*id))? {
            Some(Save::Character(save)) => Some(save),
            _ => None,
        })
    }

    /// What a character knows of `world` (nothing if they haven't been there),
    /// and of the other worlds they have been to. Being given `world` is how
    /// they come to be in it, wherever their save says they were: a server only
    /// runs one world so far. Knowledge from before `forget_before` is ignored,
    /// and saved over.
    pub fn load_character(
        &self,
        id: &CharacterId,
        world: WorldId,
    ) -> Result<(CharacterWorldSave, BTreeMap<WorldId, CharacterWorldSave>), String> {
        let save = self.character(id)?;
        let mut worlds = save.map_or_else(BTreeMap::new, |save| save.worlds);
        let here = worlds
            .remove(&world)
            .filter(|here| self.not_forgotten(here.forgotten_to, &id.0.to_string()))
            .unwrap_or_default();
        Ok((here, worlds))
    }

    /// What a character carries, which goes with them from world to world. Nothing
    /// if they are new.
    pub fn load_inventory(&self, id: &CharacterId) -> Result<Inventory, String> {
        let save = self.character(id)?;
        Ok(save.map_or_else(Inventory::default, |save| save.inventory))
    }

    /// What a character is like: `None` if they are new.
    pub fn load_sheet(&self, id: &CharacterId) -> Result<Option<Sheet>, String> {
        Ok(self.character(id)?.map(|save| save.sheet))
    }

    /// Saves a character who is in `world`, knowing `here` of it.
    pub fn save_character(
        &self,
        id: &CharacterId,
        world: WorldId,
        here: &CharacterWorldSave,
        others: &BTreeMap<WorldId, CharacterWorldSave>,
        inventory: &Inventory,
        sheet: &Sheet,
    ) {
        if let Some(queue) = &self.queue {
            let mut worlds = others.clone();
            let forgotten_to = self.forget_before;
            worlds.insert(
                world,
                CharacterWorldSave {
                    forgotten_to,
                    ..here.clone()
                },
            );
            queue.save(
                SaveKey::Character(*id),
                Save::Character(CharacterSave {
                    world,
                    worlds,
                    inventory: inventory.clone(),
                    sheet: sheet.clone(),
                }),
            );
        }
    }
}

/// Which site: a world map poi's, or a made-up one (dev tools).
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum SiteId {
    /// The site of the poi in this world map chunk.
    Poi(ChunkPos),
    /// Developer tools: a site of this kind and map seed, a building ruined
    /// this much.
    Dev { kind: PoiKind, seed: u64, ruin: u8 },
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
    /// Its clock: game minutes since it began.
    pub clock: f64,
    /// Its action clock: ticks since it began (`clock::ActionClock`).
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
}

/// The bags a character remembers on a level, by cell.
pub type RememberedBags = BTreeMap<(i64, i64), Load>;

/// Where in a site a character is.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct VisitSave {
    pub site: SiteId,
    pub level: u32,
    pub position: (i64, i64),
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

/// Seconds since 1970, for `LevelSave::last_visit`.
pub fn now() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |time| time.as_secs())
}

/// Things that have changed since they were last saved.
pub struct Unsaved<K: Ord>(BTreeSet<K>);

// By hand: deriving it would need `K: Default`.
impl<K: Ord> Default for Unsaved<K> {
    fn default() -> Self {
        Self(BTreeSet::new())
    }
}

impl<K: Ord> Unsaved<K> {
    pub fn mark(&mut self, key: K) {
        self.0.insert(key);
    }

    /// No longer marked.
    pub fn unmark(&mut self, key: &K) {
        self.0.remove(key);
    }

    /// Everything marked, forgetting it.
    pub fn take(&mut self) -> BTreeSet<K> {
        std::mem::take(&mut self.0)
    }
}

#[cfg(test)]
pub(crate) mod tests;
