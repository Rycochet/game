use super::saves::BuildingSave;
use shared::dungeon::Cell;
use shared::ground::Load;
use shared::id::Id;
use shared::item::{Inventory, Item};
use shared::poi::ChunkPos;
use shared::protocol::CharacterId;
use shared::sheet::Sheet;
use shared::site::FORGET_BEFORE;
use shared::world::WorldSeed;
use std::collections::{BTreeMap, BTreeSet};
use std::env;
use std::fs;
use std::path::PathBuf;
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
fn what_was_saved_before_forget_before_is_forgotten() {
    let folder = TestFolder::new();
    let world = WorldSeed::new(5).id();
    let mut storage = Storage::in_folder(&folder.0, world);
    let id = CharacterId(Id::for_test(1));
    let site = SiteId::Poi(ChunkPos { x: 1, y: 2 });
    let character = CharacterWorldSave {
        world_position: (3, 4),
        ..default()
    };
    let none = BTreeMap::new();
    storage.save_character(
        &id,
        world,
        &character,
        &none,
        &Inventory::default(),
        &Sheet::default(),
    );
    storage.save_site(site, &SiteSave::default());
    assert_eq!(
        storage.load_character(&id, world).unwrap().0.world_position,
        (3, 4)
    );
    assert!(storage.load_site(site).unwrap().is_some());

    storage.forget_before += 1;
    assert_eq!(
        storage.load_character(&id, world).unwrap().0.world_position,
        (0, 0)
    );
    assert_eq!(storage.load_site(site).unwrap(), None);

    // Saved again, it is remembered from now on.
    storage.save_character(
        &id,
        world,
        &character,
        &none,
        &Inventory::default(),
        &Sheet::default(),
    );
    storage.save_site(site, &SiteSave::default());
    assert_eq!(
        storage.load_character(&id, world).unwrap().0.world_position,
        (3, 4)
    );
    assert!(storage.load_site(site).unwrap().is_some());
}

#[test]
fn a_character_keeps_what_they_know_of_other_worlds() {
    let folder = TestFolder::new();
    let (first, second) = (WorldSeed::new(5).id(), WorldSeed::new(6).id());
    let id = CharacterId(Id::for_test(1));
    let at = |x| CharacterWorldSave {
        forgotten_to: FORGET_BEFORE,
        world_position: (x, 0),
        ..default()
    };
    let in_first = Storage::in_folder(&folder.0, first);
    in_first.save_character(
        &id,
        first,
        &at(1),
        &BTreeMap::new(),
        &Inventory::default(),
        &Sheet::default(),
    );
    in_first.finish_writing();
    assert!(folder.0.join(format!("characters/{}.ron", id.0)).exists());

    // Into another world: new to them, and the first is kept.
    let in_second = Storage::in_folder(&folder.0, second);
    let (here, others) = in_second.load_character(&id, second).unwrap();
    assert_eq!(here, CharacterWorldSave::default());
    assert_eq!(others, BTreeMap::from([(first, at(1))]));
    in_second.save_character(
        &id,
        second,
        &at(2),
        &others,
        &Inventory::default(),
        &Sheet::default(),
    );
    in_second.finish_writing();

    let file = fs::read_to_string(folder.0.join(format!("characters/{}.ron", id.0))).unwrap();
    let save: CharacterSave = ron::from_str(&file).unwrap();
    assert_eq!(save.world, second);
    // And back, where they were.
    assert_eq!(in_first.load_character(&id, first).unwrap().0, at(1));
}

#[test]
fn saves_are_read_back() {
    let folder = TestFolder::new();
    let world = WorldSeed::new(5).id();
    let storage = Storage::in_folder(&folder.0, world);
    let id = CharacterId(Id::for_test(1));
    assert_eq!(
        storage.load_character(&id, world).unwrap().0,
        CharacterWorldSave::default()
    );

    let site = SiteId::Poi(ChunkPos { x: -2, y: 3 });
    let character = CharacterWorldSave {
        forgotten_to: FORGET_BEFORE,
        bags: BTreeMap::from([(
            site,
            BTreeMap::from([(1, BTreeMap::from([((2, 3), Load::Full)]))]),
        )]),
        site_versions: BTreeMap::from([(site, 3)]),
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
        memory: BTreeMap::from([(site, BTreeMap::from([(1, CellMasks::from([((0, -1), 7)]))]))]),
    };
    storage.save_character(
        &id,
        world,
        &character,
        &BTreeMap::new(),
        &Inventory::default(),
        &Sheet::default(),
    );
    assert_eq!(storage.load_character(&id, world).unwrap().0, character);

    let save = SiteSave {
        forgotten_to: FORGET_BEFORE,
        version: 1,
        levels: BTreeMap::from([(
            0,
            LevelSave {
                last_visit: 99,
                mobs: BTreeMap::new(),
                unsettled: BTreeMap::new(),
                records: BTreeMap::new(),
                dead: BTreeSet::from([3]),
                ground: BTreeMap::from([(
                    (3, 4),
                    vec![Item::new(Id::for_test(1), shared::item::ItemKind::Coin)],
                )]),
                changes: BTreeMap::from([((1, 2), Cell::Floor)]),
                buildings: BTreeMap::from([(
                    4,
                    BuildingSave {
                        keeper: None,
                        trades: 2,
                        last_interaction: Some(12.5),
                        stock: Vec::new(),
                        last_made: Some(9),
                    },
                )]),
            },
        )]),
    };
    storage.save_site(site, &save);
    assert_eq!(storage.load_site(site).unwrap(), Some(save));
}

#[test]
fn broken_files_are_put_aside() {
    let folder = TestFolder::new();
    let world = WorldSeed::new(5).id();
    let storage = Storage::in_folder(&folder.0, world);
    let id = CharacterId(Id::for_test(1));
    let file = folder.0.join(format!("characters/{}.ron", id.0));
    fs::create_dir_all(file.parent().unwrap()).unwrap();
    fs::write(&file, "not a save").unwrap();
    assert_eq!(
        storage.load_character(&id, world).unwrap().0,
        CharacterWorldSave::default()
    );
    assert!(!file.exists());
    assert!(file.with_extension("ron.broken").exists());
}

#[test]
fn a_file_that_cant_be_reached_stays_and_nothing_starts_again() {
    let folder = TestFolder::new();
    let world = WorldSeed::new(5).id();
    let storage = Storage::in_folder(&folder.0, world);
    let id = CharacterId(Id::for_test(1));
    // A folder in its place can't be read as a file, whatever the system.
    let file = folder.0.join(format!("characters/{}.ron", id.0));
    fs::create_dir_all(&file).unwrap();
    assert!(storage.load_character(&id, world).is_err());
    assert!(storage.load_inventory(&id).is_err());
    assert!(file.exists());
    assert!(!file.with_extension("ron.broken").exists());
}

#[test]
fn a_file_that_isnt_text_is_broken() {
    let folder = TestFolder::new();
    let world = WorldSeed::new(5).id();
    let storage = Storage::in_folder(&folder.0, world);
    let id = CharacterId(Id::for_test(1));
    let file = folder.0.join(format!("characters/{}.ron", id.0));
    fs::create_dir_all(file.parent().unwrap()).unwrap();
    fs::write(&file, [0xff, 0xfe, 0x00]).unwrap();
    assert!(storage.load_character(&id, world).is_ok());
    assert!(file.with_extension("ron.broken").exists());
}
