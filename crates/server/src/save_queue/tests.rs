use std::collections::BTreeSet;
use std::sync::atomic::{AtomicBool, Ordering};

use shared::id::Id;
use shared::protocol::CharacterId;
use shared::world::WorldSeed;

use super::*;
use crate::accounts::{AccountId, AccountSave};
use crate::storage::tests::TestFolder;
use crate::storage::{SiteId, SiteSave};
use shared::poi::ChunkPos;

/// Saves kept in memory, which can be told to fail.
#[derive(Default)]
pub(crate) struct MemoryBackend {
    saves: Mutex<BTreeMap<SaveKey, Save>>,
    pub aside: Mutex<Vec<SaveKey>>,
    pub broken: Mutex<BTreeSet<SaveKey>>,
    pub cant_read: AtomicBool,
    pub cant_write: AtomicBool,
}

impl MemoryBackend {
    pub fn holds(&self, key: &SaveKey) -> Option<Save> {
        self.saves.lock().unwrap().get(key).cloned()
    }
}

impl SaveBackend for MemoryBackend {
    fn load(&self, key: &SaveKey) -> Result<Option<Save>, LoadError> {
        if self.cant_read.load(Ordering::Relaxed) {
            return Err(LoadError::CantReach("told not to read".into()));
        }
        if self.broken.lock().unwrap().contains(key) {
            return Err(LoadError::Broken("told it is broken".into()));
        }
        Ok(self.holds(key))
    }

    fn save(&self, key: &SaveKey, save: &Save) -> Result<(), String> {
        if self.cant_write.load(Ordering::Relaxed) {
            return Err("told not to write".into());
        }
        self.saves.lock().unwrap().insert(key.clone(), save.clone());
        Ok(())
    }

    fn remove(&self, key: &SaveKey) -> Result<(), String> {
        if self.cant_write.load(Ordering::Relaxed) {
            return Err("told not to write".into());
        }
        let mut saves = self.saves.lock().unwrap();
        match key {
            SaveKey::Sites(world) => {
                saves.retain(|saved, _| !matches!(saved, SaveKey::Site(w, _) if w == world));
            }
            _ => {
                saves.remove(key);
            }
        }
        Ok(())
    }

    fn accounts(&self) -> Result<Vec<AccountId>, String> {
        let saves = self.saves.lock().unwrap();
        Ok(saves
            .keys()
            .filter_map(|key| match key {
                SaveKey::Account(id) => Some(*id),
                _ => None,
            })
            .collect())
    }

    fn put_aside(&self, key: &SaveKey) -> Result<(), String> {
        self.broken.lock().unwrap().remove(key);
        self.saves.lock().unwrap().remove(key);
        self.aside.lock().unwrap().push(key.clone());
        Ok(())
    }
}

const SOON: Duration = Duration::from_secs(5);

fn site_key(x: i64) -> SaveKey {
    SaveKey::Site(WorldSeed::new(5).id(), SiteId::Poi(ChunkPos { x, y: 0 }))
}

fn site(version: u32) -> Save {
    Save::Site(SiteSave {
        version,
        ..SiteSave::default()
    })
}

fn memory() -> (Arc<MemoryBackend>, SaveQueue) {
    let backend = Arc::new(MemoryBackend::default());
    let queue = SaveQueue::new(backend.clone());
    (backend, queue)
}

/// What every backend must do, whichever it is.
fn behaves_as_a_backend(queue: &SaveQueue) {
    let character = SaveKey::Character(CharacterId(Id::for_test(1)));
    let account = SaveKey::Account(AccountId(Id::for_test(2)));
    let saves = [
        (site_key(1), site(1)),
        (site_key(2), site(2)),
        (account, Save::Account(AccountSave::default())),
    ];
    assert_eq!(queue.read(&character), Ok(None));
    for (key, save) in saves.clone() {
        queue.save(key, save);
    }
    assert!(queue.flush(SOON));
    for (key, save) in saves {
        assert_eq!(queue.read(&key), Ok(Some(save)));
    }

    queue.remove(site_key(1));
    queue.remove(site_key(99));
    assert!(queue.flush(SOON));
    assert_eq!(queue.read(&site_key(1)), Ok(None));
    assert!(queue.read(&site_key(2)).unwrap().is_some());

    // A site saved after they all go stays.
    queue.remove(SaveKey::Sites(WorldSeed::new(5).id()));
    queue.save(site_key(3), site(3));
    assert_eq!(queue.read(&site_key(2)), Ok(None));
    assert!(queue.flush(SOON));
    assert_eq!(queue.read(&site_key(2)), Ok(None));
    assert_eq!(queue.read(&site_key(3)), Ok(Some(site(3))));
}

#[test]
fn files_behave_as_a_backend() {
    let folder = TestFolder::new();
    behaves_as_a_backend(&SaveQueue::in_folder(&folder.0));
}

#[test]
fn memory_behaves_as_a_backend() {
    behaves_as_a_backend(&memory().1);
}

#[test]
fn a_failed_write_is_kept_and_tried_again() {
    let (backend, queue) = memory();
    backend.cant_write.store(true, Ordering::Relaxed);
    queue.save(site_key(1), site(1));
    assert!(!queue.flush(Duration::from_millis(50)));
    // Readable meanwhile, and only the latest version will be written.
    assert_eq!(queue.read(&site_key(1)), Ok(Some(site(1))));
    queue.save(site_key(1), site(2));
    assert_eq!(queue.read(&site_key(1)), Ok(Some(site(2))));
    assert_eq!(backend.holds(&site_key(1)), None);

    backend.cant_write.store(false, Ordering::Relaxed);
    assert!(queue.flush(SOON));
    assert_eq!(backend.holds(&site_key(1)), Some(site(2)));
}

#[test]
fn stopping_writes_what_is_queued() {
    let (backend, queue) = memory();
    for x in 0..100 {
        queue.save(site_key(x), site(1));
    }
    drop(queue);
    for x in 0..100 {
        assert!(backend.holds(&site_key(x)).is_some());
    }
}

#[test]
fn a_removal_that_fails_holds_back_later_sites() {
    let (backend, queue) = memory();
    queue.save(site_key(1), site(1));
    assert!(queue.flush(SOON));
    backend.cant_write.store(true, Ordering::Relaxed);
    queue.remove(SaveKey::Sites(WorldSeed::new(5).id()));
    queue.save(site_key(2), site(2));
    assert!(!queue.flush(Duration::from_millis(50)));
    backend.cant_write.store(false, Ordering::Relaxed);
    assert!(queue.flush(SOON));
    assert_eq!(backend.holds(&site_key(1)), None);
    assert_eq!(backend.holds(&site_key(2)), Some(site(2)));
}

#[test]
fn only_what_is_broken_is_put_aside() {
    let (backend, queue) = memory();
    backend.cant_read.store(true, Ordering::Relaxed);
    assert!(queue.load(&site_key(1)).is_err());
    assert!(backend.aside.lock().unwrap().is_empty());

    backend.cant_read.store(false, Ordering::Relaxed);
    backend.broken.lock().unwrap().insert(site_key(1));
    assert_eq!(queue.load(&site_key(1)), Ok(None));
    assert_eq!(*backend.aside.lock().unwrap(), [site_key(1)]);
}
