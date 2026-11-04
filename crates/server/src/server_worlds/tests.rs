use super::*;
use crate::storage::tests::TestFolder;

fn storage(folder: &TestFolder) -> Storage {
    Storage::in_folder(&folder.0, WorldSeed::new(1).id())
}

#[test]
fn other_worlds_open_beside_the_home_with_their_own_time() {
    let folder = TestFolder::new();
    let mut worlds = ServerWorlds::start(WorldSeed::new(1), storage(&folder)).unwrap();
    let other = worlds.open(WorldSeed::new(2)).unwrap();
    assert_ne!(other, worlds.home_id());
    assert_eq!(worlds.iter().count(), 2);
    let world = worlds.of(Some(other));
    world.set_time(Some(600.0), None);
    let moved = world.minutes();
    assert!((600.0..601.0).contains(&moved));
    assert!(worlds.home().minutes() < 10.0);
    // Opening it again is the same world.
    assert_eq!(worlds.open(WorldSeed::new(2)).unwrap(), other);
    assert_eq!(worlds.iter().count(), 2);
}

#[test]
fn someone_in_no_world_or_a_closed_one_is_in_the_home() {
    let mut worlds = ServerWorlds::with_home(ServerWorld::new(WorldSeed::new(1)));
    let home = worlds.home_id();
    assert_eq!(worlds.of(None).seed.id(), home);
    assert_eq!(worlds.of(Some(WorldSeed::new(9).id())).seed.id(), home);
}

#[test]
fn a_world_is_opened_from_its_save_by_its_id() {
    let folder = TestFolder::new();
    let id = WorldSeed::new(7).id();
    {
        let mut worlds = ServerWorlds::start(WorldSeed::new(1), storage(&folder)).unwrap();
        worlds.open(WorldSeed::new(7)).unwrap();
        worlds.storage.finish_writing();
    }
    let mut worlds = ServerWorlds::start(WorldSeed::new(1), storage(&folder)).unwrap();
    assert_eq!(worlds.open_saved(id), Ok(Some(id)));
    assert_eq!(worlds.get(id).unwrap().seed, WorldSeed::new(7));
    // One never saved can't be.
    assert_eq!(worlds.open_saved(WorldSeed::new(8).id()), Ok(None));
}

#[test]
fn idle_worlds_close_but_the_home_and_those_in_use_stay() {
    let mut worlds = ServerWorlds::with_home(ServerWorld::new(WorldSeed::new(1)));
    let busy = worlds.open(WorldSeed::new(2)).unwrap();
    let idle = worlds.open(WorldSeed::new(3)).unwrap();
    let in_use = BTreeSet::from([busy]);
    worlds.close_idle(&in_use);
    assert_eq!(worlds.iter().count(), 3);
    // As if it had been empty all that time.
    let long_ago = Instant::now() - WORLD_KEPT_FOR;
    worlds.empty_since.insert(idle, long_ago);
    worlds.empty_since.insert(worlds.home_id(), long_ago);
    worlds.close_idle(&in_use);
    assert!(worlds.get(idle).is_none());
    assert!(worlds.get(busy).is_some());
    assert!(worlds.get(worlds.home_id()).is_some());
}

#[test]
fn time_runs_on_by_the_server_clock_while_a_world_is_closed() {
    let folder = TestFolder::new();
    let mut worlds = ServerWorlds::start(WorldSeed::new(1), storage(&folder)).unwrap();
    let id = worlds.open(WorldSeed::new(2)).unwrap();
    worlds.of(Some(id)).set_time(Some(100.0), None);
    worlds.close(id);
    std::thread::sleep(Duration::from_millis(50));
    worlds.open_saved(id).unwrap();
    let minutes = worlds.get(id).unwrap().minutes();
    assert!(minutes > 100.0 + 0.04 && minutes < 101.0, "{minutes}");
}

#[test]
fn the_server_clock_is_saved_and_carries_on() {
    let folder = TestFolder::new();
    {
        let worlds = ServerWorlds::start(WorldSeed::new(1), storage(&folder)).unwrap();
        std::thread::sleep(Duration::from_millis(50));
        worlds.action_clock().advance(40);
        drop(worlds);
    }
    let worlds = ServerWorlds::start(WorldSeed::new(1), storage(&folder)).unwrap();
    assert!(worlds.clock().seconds() >= 0.05);
    assert_eq!(worlds.action_clock().ticks(), 40);
}
