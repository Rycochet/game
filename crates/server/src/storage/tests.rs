use super::saves::BuildingSave;
use shared::dungeon::Cell;
use shared::ground::Load;
use shared::id::Id;
use shared::item::{Inventory, Item};
use shared::poi::ChunkPos;
use shared::protocol::CharacterId;
use shared::sheet::Sheet;
use shared::site::FORGET_BEFORE;
use shared::spot::Spot;
use shared::world::WorldSeed;
use std::collections::{BTreeMap, BTreeSet};
use std::env;
use std::fs;
use std::path::PathBuf;
use std::sync::atomic::{AtomicU32, Ordering};

use super::*;
use shared::door::{Side, Swing};

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
    storage.save_character(&id, world, &character, &none, Everywhere::default());
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
    storage.save_character(&id, world, &character, &none, Everywhere::default());
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
    in_first.save_character(&id, first, &at(1), &BTreeMap::new(), Everywhere::default());
    in_first.finish_writing();
    assert!(folder.0.join(format!("characters/{}.ron", id.0)).exists());

    // Into another world: new to them, and the first is kept.
    let in_second = Storage::in_folder(&folder.0, second);
    let (here, others) = in_second.load_character(&id, second).unwrap();
    assert_eq!(here, CharacterWorldSave::default());
    assert_eq!(others, BTreeMap::from([(first, at(1))]));
    in_second.save_character(&id, second, &at(2), &others, Everywhere::default());
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
            BTreeMap::from([(1, [(Spot::new(2, 3), Load::Full)].into_iter().collect())]),
        )]),
        doors: BTreeMap::from([(
            site,
            BTreeMap::from([(1, BTreeMap::from([((4, 5), Swing::Open(Side::West))]))]),
        )]),
        site_versions: BTreeMap::from([(site, 3)]),
        left_at: BTreeMap::from([(site, (1, (5, -6)))]),
        world_position: (-40, 7),
        hall: Some(shared::spot::Spot::middle_of((12, 3))),
        dev_return: Some(crate::storage::DevReturn {
            world_position: (8, 9),
            visit: None,
            hall: Some(shared::spot::Spot::new(4, 1)),
        }),
        visit: Some(VisitSave {
            site,
            level: 1,
            position: shared::spot::Spot::new(3, -4),
            from_wilderness: true,
        }),
        pois: BTreeMap::from([(
            ChunkPos { x: 1, y: 1 },
            KnownPoi {
                known: true,
                visited: true,
                named: true,
            },
        )]),
        memory: BTreeMap::from([(site, BTreeMap::from([(1, CellMasks::from([((0, -1), 7)]))]))]),
    };
    storage.save_character(
        &id,
        world,
        &character,
        &BTreeMap::new(),
        Everywhere::default(),
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
                scents: Default::default(),
                wild_blocks: BTreeMap::from([((2, 5), 77)]),
                ground: [(
                    Spot::new(3, 4),
                    vec![Item::new(Id::for_test(1), shared::item::ItemKind::Coin)],
                )]
                .into_iter()
                .collect(),
                changes: BTreeMap::from([((1, 2), Cell::Floor)]),
                locks_placed: true,
                doors: BTreeMap::from([(
                    (6, 7),
                    shared::door::Door {
                        swing: Swing::Broken(Side::North),
                        quality: shared::item::Quality::Common,
                        material: shared::material::Material::Wood,
                        style: shared::door::DoorStyle::Plain,
                        lock: Some(shared::door::Lock {
                            id: Id::for_test(9),
                            quality: shared::item::Quality::Common,
                            locked: true,
                        }),
                        damage: 4_500,
                    },
                )]),
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

#[test]
fn a_character_keeps_the_gates_they_have_gone_through() {
    let folder = TestFolder::new();
    let world = WorldSeed::new(5).id();
    let storage = Storage::in_folder(&folder.0, world);
    let id = CharacterId(Id::for_test(1));
    assert_eq!(storage.load_gates(&id).unwrap(), BTreeSet::new());

    let gates = BTreeSet::from([3, 175_562_835]);
    storage.save_character(
        &id,
        world,
        &CharacterWorldSave::default(),
        &BTreeMap::new(),
        Everywhere {
            gates: gates.clone(),
            ..Everywhere::default()
        },
    );
    assert_eq!(storage.load_gates(&id).unwrap(), gates);
}

#[test]
fn a_save_from_before_gates_playtime_and_the_action_bar_still_loads() {
    let save = CharacterSave {
        world: WorldSeed::new(5).id(),
        worlds: BTreeMap::new(),
        inventory: Inventory::default(),
        sheet: Sheet::default(),
        gates: BTreeSet::new(),
        playtime: 0,
        action_bar: shared::ability::ActionBar::default(),
        difficulty: None,
        shown_as: None,
    };
    let text = ron::to_string(&save).unwrap();
    // Gates, playtime, the action bar and difficulty are the last fields: a
    // save from before them all ends after the sheet.
    let old = format!("{})", &text[..text.find(",gates:").unwrap()]);
    assert_ne!(old, text);
    assert_eq!(ron::from_str::<CharacterSave>(&old).unwrap(), save);
}

#[test]
fn cells_saved_before_units_load_as_their_middles() {
    let visit = VisitSave {
        site: SiteId::Wilderness,
        level: 0,
        position: shared::spot::Spot::new(500, -730),
        from_wilderness: false,
    };
    let text = ron::to_string(&visit).unwrap();
    assert_eq!(ron::from_str::<VisitSave>(&text).unwrap(), visit);
    let old = text.replace("(x:500,y:-730)", "(2,-3)");
    assert_ne!(old, text);
    let read = ron::from_str::<VisitSave>(&old).unwrap();
    assert_eq!(read.position, shared::spot::Spot::new(480, -720));

    let mut world = CharacterWorldSave {
        hall: Some(shared::spot::Spot::new(7, 9)),
        ..CharacterWorldSave::default()
    };
    let text = ron::to_string(&world).unwrap();
    assert_eq!(ron::from_str::<CharacterWorldSave>(&text).unwrap(), world);
    let old = text.replace("Some((x:7,y:9))", "Some((1,2))");
    assert_ne!(old, text);
    world.hall = Some(shared::spot::Spot::new(240, 480));
    assert_eq!(ron::from_str::<CharacterWorldSave>(&old).unwrap(), world);
}

#[test]
fn bags_saved_by_cell_load_at_their_middles() {
    let site = SiteId::Wilderness;
    let mut world = CharacterWorldSave::default();
    let bags: saves::RememberedBags = [(Spot::new(500, -730), Load::Half)].into_iter().collect();
    world.bags.insert(site, BTreeMap::from([(0, bags)]));
    let text = ron::to_string(&world).unwrap();
    assert_eq!(ron::from_str::<CharacterWorldSave>(&text).unwrap(), world);
    let old = text.replace("(x:500,y:-730)", "(2,-3)");
    assert_ne!(old, text);
    let read = ron::from_str::<CharacterWorldSave>(&old).unwrap();
    let at: Vec<Spot> = read.bags[&site][&0].keys().copied().collect();
    assert_eq!(at, [Spot::new(480, -720)]);

    let mut level = LevelSave::default();
    let coin = Item::new(Id::for_test(1), shared::item::ItemKind::Coin);
    level.ground.insert(Spot::new(500, -730), vec![coin]);
    let text = ron::to_string(&level).unwrap();
    let old = text.replace("(x:500,y:-730)", "(2,-3)");
    assert_ne!(old, text);
    let read = ron::from_str::<LevelSave>(&old).unwrap();
    assert!(read.ground.contains_key(&Spot::new(480, -720)));
}
