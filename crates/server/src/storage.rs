//! The server's saved data: where it lives, what is in each file, and
//! reading and writing them.
//!
//! Everything is in the server's own data folder, never shared with a
//! client's (see `docs/context/architecture.md`):
//!
//! - `worlds/<world id>/world.ron`: the world (`WorldSave`).
//! - `worlds/<world id>/sites/<site>.ron`: a site that has been entered
//!   (`SiteSave`).
//! - `worlds/<world id>/characters/<name>.ron`: what a character knows of
//!   the world and where they are in it (`CharacterSave`).
//!
//! Files are RON, readable by people. Fields added later must have
//! `#[serde(default)]`, so older saves still load.

use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};
use std::{env, fs, io};

use bevy::prelude::*;
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use shared::dungeon::Cell;
use shared::poi::{ChunkPos, PoiKind};
use shared::site::FORGET_BEFORE;
use shared::world::{WorldId, WorldSeed};

/// Overrides where the server keeps its data (a folder path).
pub const DATA_FOLDER_VARIABLE: &str = "HALL_OF_WORLDS_SERVER_DATA";

/// Bump when a save changes in a way older servers can't read (adding a
/// field with a default doesn't need it). A server refuses a world saved
/// with a newer format than its own.
pub const SAVE_FORMAT: u32 = 1;

/// The server's data folder: `HALL_OF_WORLDS_SERVER_DATA` if set, otherwise
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

/// Where one world's files are, or nowhere (tests: nothing is saved).
#[derive(Resource, Clone, Debug, Default)]
pub struct Storage {
    folder: Option<PathBuf>,
    /// Files saved before this are ignored (`FORGET_BEFORE`).
    forget_before: u32,
}

impl Storage {
    /// Nothing is read or written.
    pub fn none() -> Self {
        Self {
            folder: None,
            forget_before: FORGET_BEFORE,
        }
    }

    /// The files of world `id` in the data folder `data`.
    pub fn in_folder(data: &Path, id: WorldId) -> Self {
        let folder = data.join("worlds").join(format!("{:016x}", id.0));
        Self {
            folder: Some(folder),
            forget_before: FORGET_BEFORE,
        }
    }

    fn world_file(&self) -> Option<PathBuf> {
        Some(self.folder.as_ref()?.join("world.ron"))
    }

    fn site_file(&self, site: SiteId) -> Option<PathBuf> {
        let name = match site {
            // Named from when pois were called features: saved worlds have
            // these files, so the name stays.
            SiteId::Poi(chunk) => format!("feature_{}_{}.ron", chunk.x, chunk.y),
            SiteId::Dev {
                kind,
                seed,
                ruin: 0,
            } => format!("dev_{}_{seed}.ron", kind as usize),
            SiteId::Dev { kind, seed, ruin } => {
                format!("dev_{}_{seed}_ruin{ruin}.ron", kind as usize)
            }
        };
        Some(self.folder.as_ref()?.join("sites").join(name))
    }

    fn character_file(&self, name: &CharacterName) -> Option<PathBuf> {
        Some(
            self.folder
                .as_ref()?
                .join("characters")
                .join(format!("{}.ron", name.0)),
        )
    }

    /// The saved world, if there is one.
    pub fn load_world(&self) -> Result<Option<WorldSave>, String> {
        self.world_file().map_or(Ok(None), |file| read(&file))
    }

    pub fn save_world(&self, world: &WorldSave) {
        if let Some(file) = self.world_file() {
            write(&file, world);
        }
    }

    /// Whether a file saved under `forgotten_to` still counts.
    fn not_forgotten(&self, forgotten_to: u32, what: &str) -> bool {
        let kept = forgotten_to >= self.forget_before;
        if !kept {
            info!("Forgetting {what}: saved before {}", self.forget_before);
        }
        kept
    }

    /// A saved site, if it has been entered before. A broken file is put
    /// aside (and logged), so the site starts again; one from before
    /// `forget_before` is ignored, and saved over.
    pub fn load_site(&self, site: SiteId) -> Option<SiteSave> {
        let save: SiteSave = read_or_put_aside(&self.site_file(site)?)?;
        self.not_forgotten(save.forgotten_to, &format!("{site:?}"))
            .then_some(save)
    }

    pub fn save_site(&self, site: SiteId, save: &SiteSave) {
        if let Some(file) = self.site_file(site) {
            let forgotten_to = self.forget_before;
            write(
                &file,
                &SiteSave {
                    forgotten_to,
                    ..save.clone()
                },
            );
        }
    }

    /// A saved character, if they have played in this world before. A
    /// broken file is put aside (and logged), so they start again; one from
    /// before `forget_before` is ignored, and saved over.
    pub fn load_character(&self, name: &CharacterName) -> Option<CharacterSave> {
        let save: CharacterSave = read_or_put_aside(&self.character_file(name)?)?;
        self.not_forgotten(save.forgotten_to, &name.0)
            .then_some(save)
    }

    pub fn save_character(&self, name: &CharacterName, save: &CharacterSave) {
        if let Some(file) = self.character_file(name) {
            let forgotten_to = self.forget_before;
            write(
                &file,
                &CharacterSave {
                    forgotten_to,
                    ..save.clone()
                },
            );
        }
    }
}

/// A character's name: also their file name, so only letters, digits, `-`
/// and `_`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CharacterName(String);

impl CharacterName {
    pub fn new(name: &str) -> Option<Self> {
        let fine = |c: char| c.is_ascii_alphanumeric() || c == '-' || c == '_';
        (!name.is_empty() && name.len() <= 32 && name.chars().all(fine)).then(|| Self(name.into()))
    }
}

/// Which site: a world map poi's, or a made-up one (dev tools).
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum SiteId {
    /// The site of the poi in this world map chunk. (`alias`: its name in
    /// files saved when pois were called features.)
    #[serde(alias = "Feature")]
    Poi(ChunkPos),
    /// Developer tools: a site of this kind and map seed, a building ruined
    /// this much.
    Dev {
        kind: PoiKind,
        seed: u64,
        #[serde(default)]
        ruin: u8,
    },
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
    /// Its clock's whole hours (`% 24` is the time of day).
    #[serde(default)]
    pub hour: u32,
    /// Its clock: game hours since it began.
    #[serde(default)]
    pub clock: f64,
}

/// A site's file: made the first time anyone enters it.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct SiteSave {
    /// The `FORGET_BEFORE` it was saved under.
    #[serde(default)]
    pub forgotten_to: u32,
    /// The generator version it was first made with (`Generator::version`):
    /// it is always generated with this one.
    pub version: u32,
    /// Its levels that have been visited, by index.
    #[serde(default)]
    pub levels: BTreeMap<u32, LevelSave>,
}

/// A visited site level.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct LevelSave {
    /// When someone was last on it (seconds since 1970): it resets after
    /// nobody has been for a while, once there are spawns to bring back.
    #[serde(default)]
    pub last_visit: u64,
    /// Cells changed by players (dug, broken), applied over the generated
    /// level every time; a reset never undoes them.
    #[serde(default)]
    pub changes: BTreeMap<(i64, i64), Cell>,
    /// A town's buildings that players have dealt with, by `RoomId`.
    #[serde(default)]
    pub buildings: BTreeMap<u16, BuildingSave>,
}

/// What players have done with a town's building.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct BuildingSave {
    /// Trades made by players, on top of the number it was generated with.
    #[serde(default)]
    pub trades: u32,
    /// The server's clock (game hours) when last interacted with, which will
    /// decide when its stock refreshes.
    #[serde(default)]
    pub last_interaction: Option<f64>,
}

/// Cells of a level by chunk: one bit per cell (see
/// `shared::protocol::level_chunk`).
pub type CellMasks = BTreeMap<(i64, i64), u64>;

/// A character's file, for one world.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct CharacterSave {
    /// The `FORGET_BEFORE` it was saved under.
    #[serde(default)]
    pub forgotten_to: u32,
    /// Where they are on the world map (at the poi, while in a site).
    #[serde(default)]
    pub world_position: (i64, i64),
    /// The site level they are on, if any.
    #[serde(default)]
    pub visit: Option<VisitSave>,
    /// The world map chunks whose poi they have seen, and what they
    /// know of it. (`alias`: its name in files saved when pois were called
    /// features.)
    #[serde(default, alias = "features")]
    pub pois: BTreeMap<ChunkPos, KnownPoi>,
    /// The cells they have seen of each site level.
    #[serde(default)]
    pub memory: BTreeMap<SiteId, BTreeMap<u32, CellMasks>>,
}

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
    #[serde(default)]
    pub known: bool,
    /// Whether they have been inside.
    #[serde(default)]
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

    /// Everything marked, forgetting it.
    pub fn take(&mut self) -> BTreeSet<K> {
        std::mem::take(&mut self.0)
    }
}

/// Reads a file: `None` if there is none.
fn read<T: DeserializeOwned>(file: &Path) -> Result<Option<T>, String> {
    let text = match fs::read_to_string(file) {
        Ok(text) => text,
        Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(None),
        Err(error) => return Err(format!("can't read {}: {error}", file.display())),
    };
    ron::from_str(&text)
        .map(Some)
        .map_err(|error| format!("can't understand {}: {error}", file.display()))
}

/// Reads a file; one that can't be read is renamed `<name>.broken` (so it
/// isn't lost when the new one is saved) and counts as none.
fn read_or_put_aside<T: DeserializeOwned>(file: &Path) -> Option<T> {
    match read(file) {
        Ok(found) => found,
        Err(error) => {
            error!("{error}; starting it again");
            let aside = file.with_extension("ron.broken");
            if let Err(error) = fs::rename(file, &aside) {
                error!("can't move it to {}: {error}", aside.display());
            }
            None
        }
    }
}

/// Writes a file (and its folder), through a temporary file so a crash
/// never leaves half of one. Failures are logged: the game goes on.
fn write<T: Serialize>(file: &Path, value: &T) {
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
    if let Err(error) = result {
        error!("Can't save {}: {error}", file.display());
    }
}

#[cfg(test)]
pub(crate) mod tests {
    use std::sync::atomic::{AtomicU32, Ordering};

    use super::*;

    /// A new, empty data folder for one test (in the system's temporary
    /// folder), removed when dropped.
    pub(crate) struct TestFolder(pub PathBuf);

    impl TestFolder {
        pub fn new() -> Self {
            static NEXT: AtomicU32 = AtomicU32::new(0);
            let n = NEXT.fetch_add(1, Ordering::Relaxed);
            let folder =
                env::temp_dir().join(format!("hall-of-worlds-test-{}-{n}", std::process::id()));
            let _ = fs::remove_dir_all(&folder);
            Self(folder)
        }
    }

    impl Drop for TestFolder {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.0);
        }
    }

    #[test]
    fn characters_saved_when_pois_were_features_still_load() {
        let text = "(
            visit: Some((site: Feature((x: -2, y: 3)), level: 1, position: (3, -4))),
            features: {(x: 1, y: 1): (visited: true)},
            memory: {Feature((x: -2, y: 3)): {}},
        )";
        let site = SiteId::Poi(ChunkPos { x: -2, y: 3 });
        let save: CharacterSave = ron::from_str(text).unwrap();
        assert_eq!(save.visit.unwrap().site, site);
        assert!(save.pois[&ChunkPos { x: 1, y: 1 }].visited);
        assert!(save.memory.contains_key(&site));
    }

    #[test]
    fn what_was_saved_before_forget_before_is_forgotten() {
        let folder = TestFolder::new();
        let mut storage = Storage::in_folder(&folder.0, WorldId(5));
        let name = CharacterName::new("someone").unwrap();
        let site = SiteId::Poi(ChunkPos { x: 1, y: 2 });
        storage.save_character(&name, &CharacterSave::default());
        storage.save_site(site, &SiteSave::default());
        assert!(storage.load_character(&name).is_some());
        assert!(storage.load_site(site).is_some());

        storage.forget_before += 1;
        assert_eq!(storage.load_character(&name), None);
        assert_eq!(storage.load_site(site), None);

        // Saved again, it is remembered from now on.
        storage.save_character(&name, &CharacterSave::default());
        storage.save_site(site, &SiteSave::default());
        assert!(storage.load_character(&name).is_some());
        assert!(storage.load_site(site).is_some());
    }

    #[test]
    fn saves_are_read_back() {
        let folder = TestFolder::new();
        let storage = Storage::in_folder(&folder.0, WorldId(5));
        let name = CharacterName::new("someone").unwrap();
        assert_eq!(storage.load_character(&name), None);

        let site = SiteId::Poi(ChunkPos { x: -2, y: 3 });
        let character = CharacterSave {
            forgotten_to: FORGET_BEFORE,
            world_position: (-40, 7),
            visit: Some(VisitSave {
                site,
                level: 1,
                position: (3, -4),
            }),
            pois: BTreeMap::from([(
                ChunkPos { x: 1, y: 1 },
                KnownPoi {
                    known: true,
                    visited: true,
                },
            )]),
            memory: BTreeMap::from([(
                site,
                BTreeMap::from([(1, CellMasks::from([((0, -1), 7)]))]),
            )]),
        };
        storage.save_character(&name, &character);
        assert_eq!(storage.load_character(&name), Some(character));

        let save = SiteSave {
            forgotten_to: FORGET_BEFORE,
            version: 1,
            levels: BTreeMap::from([(
                0,
                LevelSave {
                    last_visit: 99,
                    changes: BTreeMap::from([((1, 2), Cell::Floor)]),
                    buildings: BTreeMap::from([(
                        4,
                        BuildingSave {
                            trades: 2,
                            last_interaction: Some(12.5),
                        },
                    )]),
                },
            )]),
        };
        storage.save_site(site, &save);
        assert_eq!(storage.load_site(site), Some(save));
    }

    #[test]
    fn missing_fields_take_their_defaults() {
        let save: CharacterSave = ron::from_str("(world_position: (1, 2))").unwrap();
        assert_eq!(save.world_position, (1, 2));
        assert!(save.visit.is_none() && save.memory.is_empty());
    }

    #[test]
    fn broken_files_are_put_aside() {
        let folder = TestFolder::new();
        let storage = Storage::in_folder(&folder.0, WorldId(5));
        let name = CharacterName::new("broken").unwrap();
        let file = storage.character_file(&name).unwrap();
        fs::create_dir_all(file.parent().unwrap()).unwrap();
        fs::write(&file, "not a save").unwrap();
        assert_eq!(storage.load_character(&name), None);
        assert!(!file.exists());
        assert!(file.with_extension("ron.broken").exists());
    }

    #[test]
    fn character_names_are_safe_file_names() {
        assert!(CharacterName::new("player_1").is_some());
        for bad in ["", "../x", "a b", "a/b", "C:"] {
            assert!(CharacterName::new(bad).is_none(), "{bad}");
        }
    }
}
