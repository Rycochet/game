//! The server's saved data: what is in each save, and loading and saving them.
//! Where they are kept is behind `saves::SaveBackend`, and writing them goes
//! through the save thread (`save_queue`).
//!
//! Everything is in the server's own data folder, never shared with a
//! client's (see `docs/context/architecture.md`), as RON files, readable by
//! people. Fields added later must have `#[serde(default)]`, so older saves
//! still load.

use std::env;
#[cfg(test)]
use std::path::Path;
use std::path::PathBuf;

use bevy::prelude::*;
use shared::site::FORGET_BEFORE;
use shared::world::WorldId;

use crate::save_queue::{FINISH_WITHIN, SaveQueue};
use crate::saves::{LoadError, Save, SaveKey};

// Split into files (`cargo xtask lint` keeps files small).
mod characters;
mod roads;
mod saves;
mod sites;
mod unsaved;

pub use characters::Everywhere;
pub use roads::RoadSave;
pub use saves::{
    CellMasks, CharacterSave, CharacterWorldSave, DevReturn, KnownPoi, LevelSave, MobRecord,
    RememberedDoors, SavedMob, ServerSave, SiteId, SiteSave, VisitSave, WorldSave,
};
pub use unsaved::{Unsaved, now};

/// Overrides where the server keeps its data (a folder path).
pub const DATA_FOLDER_VARIABLE: &str = "HOW_DATA";

/// Bump when a save changes in a way older servers can't read (adding a
/// field with a default doesn't need it). A server refuses a world saved
/// with a newer format than its own.
pub const SAVE_FORMAT: u32 = 4;

/// Saves from before this format are moved out of the way on starting
/// (`saves::set_aside_old_saves`): raising it wipes every server's saves.
pub const OLDEST_SAVE_FORMAT: u32 = 4;

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
    /// How many handles there are on its save queue (tests).
    #[cfg(test)]
    pub(crate) fn queue_handles(&self) -> usize {
        self.queue.as_ref().map_or(0, SaveQueue::handles)
    }

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

    /// The same saves, for world `id` (the characters' are the same for all).
    pub(crate) fn for_world(&self, id: WorldId) -> Self {
        Self {
            world: id,
            ..self.clone()
        }
    }

    /// The server's own clocks, if saved. An error if they can't be read.
    pub fn load_server(&self) -> Result<Option<ServerSave>, String> {
        let Some(queue) = &self.queue else {
            return Ok(None);
        };
        match queue.read(&SaveKey::Server) {
            Ok(Some(Save::Server(save))) => Ok(Some(save)),
            Ok(_) => Ok(None),
            Err(LoadError::CantReach(error) | LoadError::Broken(error)) => Err(error),
        }
    }

    pub fn save_server(&self, save: &ServerSave) {
        if let Some(queue) = &self.queue {
            queue.save(SaveKey::Server, Save::Server(save.clone()));
        }
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

    /// The seed of the world saved here: `None` if there is none, or it makes
    /// no sense. An error only if it couldn't be reached, as it may be fine.
    pub(crate) fn saved_world_seed(&self) -> Result<Option<shared::world::WorldSeed>, String> {
        let Some(queue) = &self.queue else {
            return Ok(None);
        };
        match queue.read(&SaveKey::World(self.world)) {
            Ok(Some(Save::World(save))) => Ok(Some(save.seed)),
            Ok(_) | Err(LoadError::Broken(_)) => Ok(None),
            Err(LoadError::CantReach(error)) => Err(error),
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
}

#[cfg(test)]
pub(crate) mod tests;
