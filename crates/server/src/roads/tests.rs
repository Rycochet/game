use std::time::{Duration, Instant};

use shared::poi::ChunkPos;
use shared::road::ROAD_VERSION;

use super::*;
use crate::storage::tests::TestFolder;

fn seed() -> WorldSeed {
    WorldSeed::new(1)
}

/// Everything `worker` finds of an area 12 chunks across, waiting until it
/// has been quiet for a while.
fn found_by(worker: &RoadWorker) -> Vec<Arc<Road>> {
    worker.ask((0, -400), (800, 400));
    let start = Instant::now();
    let mut found = Vec::new();
    let mut quiet_since = Instant::now();
    while start.elapsed() < Duration::from_secs(60) {
        let new = worker.found();
        if new.is_empty() {
            if !found.is_empty() && quiet_since.elapsed() > Duration::from_secs(2) {
                break;
            }
        } else {
            quiet_since = Instant::now();
        }
        found.extend(new);
        thread::sleep(Duration::from_millis(20));
    }
    found
}

#[test]
fn the_thread_finds_the_roads_of_an_area_and_saves_them() {
    let folder = TestFolder::new();
    let storage = Storage::in_folder(&folder.0, seed().id());
    let found = found_by(&RoadWorker::start(seed(), storage.clone()));
    assert!(!found.is_empty());
    storage.finish_writing();
    let road = &found[0];
    assert_eq!(storage.load_road(road.link), Some(Some(Road::clone(road))));
    // A second server reads them back rather than working them out.
    let again = found_by(&RoadWorker::start(seed(), storage));
    assert_eq!(again, found);
}

#[test]
fn roads_from_another_version_are_worked_out_again() {
    let folder = TestFolder::new();
    let storage = Storage::in_folder(&folder.0, seed().id());
    let link = RoadLink::new(ChunkPos { x: 1, y: -4 }, ChunkPos { x: 2, y: -4 });
    storage.save_road(link, None);
    storage.finish_writing();
    assert_eq!(storage.load_road(link), Some(None));
    let file = folder
        .0
        .join(format!("worlds/{}/roads/1_-4_2_-4.ron", seed().id().0));
    let text = std::fs::read_to_string(&file).expect("saved");
    let older = text.replace(&format!("version: {ROAD_VERSION}"), "version: 0");
    assert_ne!(older, text);
    std::fs::write(&file, older).expect("written");
    assert_eq!(storage.load_road(link), None);
}

#[test]
fn dropping_the_worker_lets_go_of_the_saves_at_once() {
    let folder = TestFolder::new();
    let storage = Storage::in_folder(&folder.0, seed().id());
    let worker = RoadWorker::start(seed(), storage.clone());
    assert_eq!(storage.queue_handles(), 2);
    // Busy working out roads when it is dropped.
    worker.ask((0, -2000), (4000, 2000));
    drop(worker);
    // Only this one is left, so dropping it would finish writing straight away.
    assert_eq!(storage.queue_handles(), 1);
}
