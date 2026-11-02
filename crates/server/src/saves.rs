//! Where saves are kept, behind `SaveBackend`: the game hands over and gets back
//! whole save structs, and never knows how they are stored. Only files so far;
//! `docs/context/postgres-migration.md` is the plan for a second.

use std::path::{Path, PathBuf};
use std::{fs, io};

use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use shared::protocol::CharacterId;
use shared::world::WorldId;
use time::OffsetDateTime;

use crate::accounts::{AccountId, AccountSave, LoginSave};
use crate::storage::{CharacterSave, OLDEST_SAVE_FORMAT, SAVE_FORMAT, SiteId, SiteSave, WorldSave};

/// Says which format a data folder's saves are in.
const FORMAT_FILE: &str = "save_format.ron";

/// What holds saves in a data folder, moved together at a wipe. Nothing else there
/// is touched: a server's certificates (`tls/`) outlive its saves.
const SAVE_FOLDERS: [&str; 4] = ["accounts", "characters", "logins", "worlds"];

/// Which save.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) enum SaveKey {
    World(WorldId),
    /// All of a world's sites at once: only ever removed.
    Sites(WorldId),
    Site(WorldId, SiteId),
    Character(CharacterId),
    Account(AccountId),
    /// By the hash of the login.
    Login(String),
}

/// A whole save. Only passed along to be written, so its size doesn't matter.
#[allow(clippy::large_enum_variant)]
#[derive(Clone, Debug, PartialEq)]
pub(crate) enum Save {
    World(WorldSave),
    Site(SiteSave),
    Character(CharacterSave),
    Account(AccountSave),
    Login(LoginSave),
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
}

/// Saves as RON files in a data folder:
///
/// - `worlds/<world id>/world.ron`: the world.
/// - `worlds/<world id>/sites/<site>.ron`: a site that has been entered.
/// - `characters/<id>.ron`: a character, who can move between worlds.
/// - `accounts/<id>.ron`: an account and its characters.
/// - `logins/<hash>.ron`: which account a way of logging in belongs to.
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
            SaveKey::World(id) => self.world_folder(*id).join("world.ron"),
            SaveKey::Sites(id) => self.world_folder(*id).join("sites"),
            SaveKey::Site(id, site) => self
                .world_folder(*id)
                .join("sites")
                .join(site_file_name(*site)),
            SaveKey::Character(id) => self.data.join("characters").join(format!("{}.ron", id.0)),
            SaveKey::Account(id) => self.data.join("accounts").join(format!("{}.ron", id.0)),
            SaveKey::Login(hash) => self.data.join("logins").join(format!("{hash}.ron")),
        }
    }
}

fn site_file_name(site: SiteId) -> String {
    match site {
        SiteId::Poi(chunk) => format!("poi_{}_{}.ron", chunk.x, chunk.y),
        SiteId::Dev {
            kind,
            seed,
            ruin: 0,
        } => format!("dev_{}_{seed}.ron", kind as usize),
        SiteId::Dev { kind, seed, ruin } => {
            format!("dev_{}_{seed}_ruin{ruin}.ron", kind as usize)
        }
    }
}

impl SaveBackend for FileBackend {
    fn load(&self, key: &SaveKey) -> Result<Option<Save>, LoadError> {
        let file = self.path(key);
        Ok(match key {
            SaveKey::World(_) => read(&file)?.map(Save::World),
            SaveKey::Sites(_) => None,
            SaveKey::Site(..) => read(&file)?.map(Save::Site),
            SaveKey::Character(_) => read(&file)?.map(Save::Character),
            SaveKey::Account(_) => read(&file)?.map(Save::Account),
            SaveKey::Login(_) => read(&file)?.map(Save::Login),
        })
    }

    fn save(&self, key: &SaveKey, save: &Save) -> Result<(), String> {
        let file = self.path(key);
        match save {
            Save::World(save) => write(&file, save),
            Save::Site(save) => write(&file, save),
            Save::Character(save) => write(&file, save),
            Save::Account(save) => write(&file, save),
            Save::Login(save) => write(&file, save),
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
}

/// `save_format.ron`.
#[derive(Serialize, Deserialize)]
struct FormatSave {
    format: u32,
}

/// Run before anything opens a data folder. If its saves are from before
/// `OLDEST_SAVE_FORMAT`, moves them into a dated folder beside them, for the owner
/// to delete: nothing is ever deleted here. Folders without a format file are from
/// before there was one. An error if they can't be moved, as the server would then
/// be mixing old saves with new.
pub(crate) fn set_aside_old_saves(data: &Path) -> Result<(), String> {
    let file = data.join(FORMAT_FILE);
    // One that makes no sense says nothing, the same as there being none.
    let format = read::<FormatSave>(&file)
        .unwrap_or(None)
        .map(|save| save.format);
    if format.is_none_or(|format| format < OLDEST_SAVE_FORMAT) {
        let found: Vec<_> = SAVE_FOLDERS
            .iter()
            .filter(|folder| data.join(folder).exists())
            .collect();
        if !found.is_empty() {
            let aside = old_saves_folder(data);
            fs::create_dir_all(&aside)
                .map_err(|error| format!("can't make {}: {error}", aside.display()))?;
            for folder in found {
                fs::rename(data.join(folder), aside.join(folder))
                    .map_err(|error| format!("can't move {folder} aside: {error}"))?;
            }
            // Logging may not have started yet, and a host needs to see this.
            eprintln!(
                "Saves from an older version can't be used, so they were moved to {}",
                aside.display()
            );
        }
    }
    // Never lowered: a newer server's saves are refused later, with a reason.
    if format.is_none_or(|format| format < SAVE_FORMAT) {
        let format = FormatSave {
            format: SAVE_FORMAT,
        };
        write(&file, &format)?;
    }
    Ok(())
}

/// `old-saves-<today>` in the data folder, numbered if there already is one.
fn old_saves_folder(data: &Path) -> PathBuf {
    let today = OffsetDateTime::now_utc().date();
    let name = format!(
        "old-saves-{}-{:02}-{:02}",
        today.year(),
        today.month() as u8,
        today.day()
    );
    let mut folder = data.join(&name);
    let mut number = 1;
    while folder.exists() {
        number += 1;
        folder = data.join(format!("{name}-{number}"));
    }
    folder
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
mod tests {
    use super::*;
    use crate::storage::tests::TestFolder;

    /// A data folder as an older server left it, with some certificates.
    fn old_folder() -> TestFolder {
        let folder = TestFolder::new();
        for file in [
            "accounts/a.ron",
            "characters/player.ron",
            "logins/l.ron",
            "worlds/00112233/world.ron",
            "tls/key.pem",
        ] {
            let file = folder.0.join(file);
            fs::create_dir_all(file.parent().unwrap()).unwrap();
            fs::write(file, "kept").unwrap();
        }
        folder
    }

    fn set_aside_folders(data: &Path) -> Vec<PathBuf> {
        fs::read_dir(data)
            .unwrap()
            .map(|entry| entry.unwrap().path())
            .filter(|path| {
                let name = path.file_name().unwrap().to_string_lossy();
                name.starts_with("old-saves-")
            })
            .collect()
    }

    #[test]
    fn old_saves_are_moved_aside_with_nothing_lost_and_certificates_left() {
        let folder = old_folder();
        set_aside_old_saves(&folder.0).unwrap();
        let aside = set_aside_folders(&folder.0);
        assert_eq!(aside.len(), 1);
        for file in [
            "accounts/a.ron",
            "characters/player.ron",
            "logins/l.ron",
            "worlds/00112233/world.ron",
        ] {
            assert_eq!(fs::read_to_string(aside[0].join(file)).unwrap(), "kept");
            assert!(!folder.0.join(file).exists(), "{file}");
        }
        assert!(folder.0.join("tls/key.pem").exists());
        assert!(!aside[0].join("tls").exists());
    }

    #[test]
    fn saves_in_the_current_format_stay() {
        let folder = old_folder();
        set_aside_old_saves(&folder.0).unwrap();
        fs::create_dir_all(folder.0.join("characters")).unwrap();
        set_aside_old_saves(&folder.0).unwrap();
        assert_eq!(set_aside_folders(&folder.0).len(), 1);
        assert!(folder.0.join("characters").exists());
    }

    #[test]
    fn an_older_format_is_moved_aside_too_and_each_wipe_gets_its_own_folder() {
        let folder = old_folder();
        set_aside_old_saves(&folder.0).unwrap();
        fs::create_dir_all(folder.0.join("worlds")).unwrap();
        let older = format!("(format: {})", OLDEST_SAVE_FORMAT - 1);
        fs::write(folder.0.join(FORMAT_FILE), older).unwrap();
        set_aside_old_saves(&folder.0).unwrap();
        assert_eq!(set_aside_folders(&folder.0).len(), 2);
        assert!(!folder.0.join("worlds").exists());
    }

    #[test]
    fn a_newer_format_is_left_for_the_world_to_refuse() {
        let folder = old_folder();
        let newer = format!("(format: {})", SAVE_FORMAT + 1);
        fs::write(folder.0.join(FORMAT_FILE), &newer).unwrap();
        set_aside_old_saves(&folder.0).unwrap();
        assert!(set_aside_folders(&folder.0).is_empty());
        assert_eq!(
            fs::read_to_string(folder.0.join(FORMAT_FILE)).unwrap(),
            newer
        );
    }

    #[test]
    fn a_new_folder_just_notes_its_format() {
        let folder = TestFolder::new();
        fs::create_dir_all(&folder.0).unwrap();
        set_aside_old_saves(&folder.0).unwrap();
        assert!(set_aside_folders(&folder.0).is_empty());
        assert!(folder.0.join(FORMAT_FILE).exists());
    }
}
