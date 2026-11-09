//! Where saves are kept, behind `SaveBackend`: the game hands over and gets back
//! whole save structs, and never knows how they are stored. Only files so far;
//! `docs/context/postgres-migration.md` is the plan for a second.

use std::path::{Path, PathBuf};
use std::{fs, io};

use serde::Serialize;
use serde::de::DeserializeOwned;
use shared::protocol::CharacterId;
use shared::road::RoadLink;
use shared::world::WorldId;

mod old_saves;
mod site_file;
pub(crate) use old_saves::set_aside_old_saves;
use site_file::site_file_name;

use crate::accounts::{AccountId, AccountSave, LoginSave, NameIndex};
use crate::storage::{CharacterSave, RoadSave, ServerSave, SiteId, SiteSave, WorldSave};

/// Says which format a data folder's saves are in.
const FORMAT_FILE: &str = "save_format.ron";

/// What holds saves in a data folder, moved together at a wipe. Nothing else there
/// is touched: a server's certificates (`tls/`) outlive its saves.
const SAVE_FOLDERS: [&str; 4] = ["accounts", "characters", "logins", "worlds"];

/// Which save.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) enum SaveKey {
    /// The server's own clocks.
    Server,
    World(WorldId),
    /// All of a world's sites at once: only ever removed.
    Sites(WorldId),
    Site(WorldId, SiteId),
    /// A road worked out once, kept so it needn't be again.
    Road(WorldId, RoadLink),
    Character(CharacterId),
    Account(AccountId),
    /// By the hash of the login.
    Login(String),
    /// Every character's name, so no two are the same.
    Names,
}

/// A whole save. Only passed along to be written, so its size doesn't matter.
#[allow(clippy::large_enum_variant)]
#[derive(Clone, Debug, PartialEq)]
pub(crate) enum Save {
    Server(ServerSave),
    World(WorldSave),
    Site(SiteSave),
    Road(RoadSave),
    Character(CharacterSave),
    Account(AccountSave),
    Login(LoginSave),
    Names(NameIndex),
}

/// Why a save couldn't be loaded. The difference matters: only something that
/// can't be understood may be put aside and started again. Starting again
/// because it couldn't be reached would save a blank character over a good one.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum LoadError {
    /// It couldn't be reached, for now at least, and may be fine.
    CantReach(String),
    /// It was reached, and makes no sense.
    Broken(String),
}

/// Somewhere to keep saves. `Send + Sync` means it can be used from several
/// threads at once: the save thread writes while the game reads.
pub(crate) trait SaveBackend: Send + Sync {
    /// The save, or `None` if there isn't one.
    fn load(&self, key: &SaveKey) -> Result<Option<Save>, LoadError>;

    /// Writes a save whole, so a crash never leaves half of one.
    fn save(&self, key: &SaveKey, save: &Save) -> Result<(), String>;

    /// Removes a save, or all of a world's sites. Not there is not a failure.
    fn remove(&self, key: &SaveKey) -> Result<(), String>;

    /// Keeps a broken save somewhere it won't be loaded or saved over.
    fn put_aside(&self, key: &SaveKey) -> Result<(), String>;

    /// Every account saved (only for making the name index, once).
    fn accounts(&self) -> Result<Vec<AccountId>, String>;
}

/// Saves as RON files in a data folder:
///
/// - `worlds/<world id>/world.ron`: the world.
/// - `worlds/<world id>/sites/<site>.ron`: a site that has been entered.
/// - `worlds/<world id>/roads/<a>_<b>.ron`: a road between the towns in chunks `a` and `b`.
/// - `characters/<id>.ron`: a character, who can move between worlds.
/// - `accounts/<id>.ron`: an account and its characters.
/// - `logins/<hash>.ron`: which account a way of logging in belongs to.
/// - `accounts/names.ron`: every character's name (moved with the accounts).
pub(crate) struct FileBackend {
    data: PathBuf,
}

impl FileBackend {
    pub fn new(data: &Path) -> Self {
        Self {
            data: data.to_owned(),
        }
    }

    fn world_folder(&self, id: WorldId) -> PathBuf {
        self.data.join("worlds").join(id.0.to_string())
    }

    /// The file a save is in (for `Sites`, their folder).
    fn path(&self, key: &SaveKey) -> PathBuf {
        match key {
            SaveKey::Server => self.data.join("server.ron"),
            SaveKey::World(id) => self.world_folder(*id).join("world.ron"),
            SaveKey::Sites(id) => self.world_folder(*id).join("sites"),
            SaveKey::Site(id, site) => self
                .world_folder(*id)
                .join("sites")
                .join(site_file_name(*site)),
            SaveKey::Road(id, link) => self.world_folder(*id).join("roads").join(format!(
                "{}_{}_{}_{}.ron",
                link.a.x, link.a.y, link.b.x, link.b.y
            )),
            SaveKey::Character(id) => self.data.join("characters").join(format!("{}.ron", id.0)),
            SaveKey::Account(id) => self.data.join("accounts").join(format!("{}.ron", id.0)),
            SaveKey::Login(hash) => self.data.join("logins").join(format!("{hash}.ron")),
            SaveKey::Names => self.data.join("accounts").join("names.ron"),
        }
    }
}

impl SaveBackend for FileBackend {
    fn load(&self, key: &SaveKey) -> Result<Option<Save>, LoadError> {
        let file = self.path(key);
        Ok(match key {
            SaveKey::Server => read(&file)?.map(Save::Server),
            SaveKey::World(_) => read(&file)?.map(Save::World),
            SaveKey::Sites(_) => None,
            SaveKey::Site(..) => read(&file)?.map(Save::Site),
            SaveKey::Road(..) => read(&file)?.map(Save::Road),
            SaveKey::Character(_) => read(&file)?.map(Save::Character),
            SaveKey::Account(_) => read(&file)?.map(Save::Account),
            SaveKey::Login(_) => read(&file)?.map(Save::Login),
            SaveKey::Names => read(&file)?.map(Save::Names),
        })
    }

    fn save(&self, key: &SaveKey, save: &Save) -> Result<(), String> {
        let file = self.path(key);
        match save {
            Save::Server(save) => write(&file, save),
            Save::World(save) => write(&file, save),
            Save::Site(save) => write(&file, save),
            Save::Road(save) => write(&file, save),
            Save::Character(save) => write(&file, save),
            Save::Account(save) => write(&file, save),
            Save::Login(save) => write(&file, save),
            Save::Names(save) => write(&file, save),
        }
    }

    fn remove(&self, key: &SaveKey) -> Result<(), String> {
        let path = self.path(key);
        let result = match key {
            SaveKey::Sites(_) => fs::remove_dir_all(&path),
            _ => fs::remove_file(&path),
        };
        match result {
            Err(error) if error.kind() != io::ErrorKind::NotFound => {
                Err(format!("can't delete {}: {error}", path.display()))
            }
            _ => Ok(()),
        }
    }

    fn put_aside(&self, key: &SaveKey) -> Result<(), String> {
        let file = self.path(key);
        let aside = file.with_extension("ron.broken");
        fs::rename(&file, &aside).map_err(|error| format!("can't move it aside: {error}"))
    }

    fn accounts(&self) -> Result<Vec<AccountId>, String> {
        let folder = self.data.join("accounts");
        let entries = match fs::read_dir(&folder) {
            Ok(entries) => entries,
            Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(Vec::new()),
            Err(error) => return Err(format!("can't read {}: {error}", folder.display())),
        };
        let mut found = Vec::new();
        for entry in entries {
            let path = entry.map_err(|error| error.to_string())?.path();
            // Account files are named by their id; anything else isn't one.
            let id = path
                .file_name()
                .and_then(|name| name.to_str()?.strip_suffix(".ron"));
            if let Some(id) = id.and_then(|id| uuid::Uuid::parse_str(id).ok()) {
                found.push(AccountId(shared::id::Id(id)));
            }
        }
        Ok(found)
    }
}

/// Reads a file: `None` if there is none.
fn read<T: DeserializeOwned>(file: &Path) -> Result<Option<T>, LoadError> {
    let text = match fs::read_to_string(file) {
        Ok(text) => text,
        Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(None),
        // Read, but not text.
        Err(error) if error.kind() == io::ErrorKind::InvalidData => {
            return Err(LoadError::Broken(format!(
                "can't understand {}: {error}",
                file.display()
            )));
        }
        Err(error) => {
            return Err(LoadError::CantReach(format!(
                "can't read {}: {error}",
                file.display()
            )));
        }
    };
    ron::from_str(&text)
        .map(Some)
        .map_err(|error| LoadError::Broken(format!("can't understand {}: {error}", file.display())))
}

/// Writes a file (and its folder), through a temporary file so a crash never
/// leaves half of one.
fn write<T: Serialize>(file: &Path, value: &T) -> Result<(), String> {
    let result = (|| {
        // Nested no deeper than a line per map entry.
        let pretty = ron::ser::PrettyConfig::default().depth_limit(2);
        let text = ron::ser::to_string_pretty(value, pretty).map_err(io::Error::other)?;
        if let Some(folder) = file.parent() {
            fs::create_dir_all(folder)?;
        }
        let temporary = file.with_extension("ron.new");
        fs::write(&temporary, text)?;
        fs::rename(&temporary, file)
    })();
    result.map_err(|error| format!("can't write {}: {error}", file.display()))
}

#[cfg(test)]
mod tests;
