//! The save thread: everything saved goes through here, so a slow or failing
//! `SaveBackend` never holds up the game, and nothing is lost when a write fails.
//!
//! Only the latest version of each save is kept, and stays queued until it has
//! really been written, however many tries that takes. Loading looks in the
//! queue first, so what was just saved is what is read back: reconnecting saves
//! the old connection's character and loads it straight away.

use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;
use std::sync::{Arc, Condvar, Mutex};
use std::thread::{self, JoinHandle};
use std::time::Duration;

use bevy::prelude::*;

use crate::saves::{FileBackend, LoadError, Save, SaveBackend, SaveKey};

/// How long stopping waits for saves to be written. Under the 10 seconds Docker
/// gives a stopping container before killing it.
pub(crate) const FINISH_WITHIN: Duration = Duration::from_secs(8);

/// How long after a failed write to try again, doubling up to `LONGEST_RETRY`.
/// A new save arriving tries everything again sooner.
const FIRST_RETRY: Duration = Duration::from_secs(1);
const LONGEST_RETRY: Duration = Duration::from_secs(30);

/// A change waiting to be written.
#[derive(Clone)]
enum Change {
    // In an `Arc` (a shared pointer) so that writing one needs no copy of it.
    Save(Arc<Save>),
    Remove,
}

#[derive(Clone)]
struct Pending {
    /// Numbered in the order asked for, which is the order they are written in.
    number: u64,
    change: Change,
}

#[derive(Default)]
struct State {
    pending: BTreeMap<SaveKey, Pending>,
    last_number: u64,
    /// The thread should try once more, not waiting out its retry.
    finishing: bool,
    /// The thread should end, written or not.
    stop: bool,
}

/// What the game's threads and the save thread share.
struct Shared {
    backend: Arc<dyn SaveBackend>,
    state: Mutex<State>,
    /// Tells anyone waiting that `state` has changed.
    changed: Condvar,
}

impl Shared {
    fn lock(&self) -> std::sync::MutexGuard<'_, State> {
        self.state.lock().expect("never poisoned")
    }
}

struct Worker {
    shared: Arc<Shared>,
    thread: Mutex<Option<JoinHandle<()>>>,
}

/// A handle on the save thread. Cloning it gives another handle on the same
/// thread; when the last one is dropped, the thread finishes writing and ends.
#[derive(Clone)]
pub(crate) struct SaveQueue(Arc<Worker>);

impl SaveQueue {
    /// Saving in files in the data folder `data`.
    pub fn in_folder(data: &Path) -> Self {
        Self::new(Arc::new(FileBackend::new(data)))
    }

    pub fn new(backend: Arc<dyn SaveBackend>) -> Self {
        let shared = Arc::new(Shared {
            backend,
            state: Mutex::default(),
            changed: Condvar::new(),
        });
        let for_thread = shared.clone();
        let thread = thread::Builder::new()
            .name("saves".into())
            .spawn(move || run(&for_thread))
            .expect("the save thread can start");
        Self(Arc::new(Worker {
            shared,
            thread: Mutex::new(Some(thread)),
        }))
    }

    /// Queues a save, replacing any older version waiting to be written.
    pub fn save(&self, key: SaveKey, save: Save) {
        self.queue(key, Change::Save(Arc::new(save)));
    }

    /// Queues removing a save, or all of a world's sites.
    pub fn remove(&self, key: SaveKey) {
        self.queue(key, Change::Remove);
    }

    fn queue(&self, key: SaveKey, change: Change) {
        let shared = &self.0.shared;
        let mut state = shared.lock();
        if let SaveKey::Sites(world) = key {
            // Any of its sites waiting to be written are going anyway.
            state
                .pending
                .retain(|queued, _| !matches!(queued, SaveKey::Site(w, _) if *w == world));
        }
        state.last_number += 1;
        let number = state.last_number;
        state.pending.insert(key, Pending { number, change });
        shared.changed.notify_all();
    }

    /// A save as it now stands, whether or not it has been written yet.
    pub fn read(&self, key: &SaveKey) -> Result<Option<Save>, LoadError> {
        let shared = &self.0.shared;
        {
            let state = shared.lock();
            if let Some(pending) = state.pending.get(key) {
                return Ok(match &pending.change {
                    Change::Save(save) => Some(Save::clone(save)),
                    Change::Remove => None,
                });
            }
            if let SaveKey::Site(world, _) = key
                && state.pending.contains_key(&SaveKey::Sites(*world))
            {
                return Ok(None);
            }
        }
        shared.backend.load(key)
    }

    /// `read`, but a broken save is put aside (and logged), and counts as none so
    /// that it starts again. An error if it couldn't be reached, or put aside:
    /// starting again then could save over something good.
    pub fn load(&self, key: &SaveKey) -> Result<Option<Save>, String> {
        match self.read(key) {
            Ok(found) => Ok(found),
            Err(LoadError::CantReach(error)) => Err(error),
            Err(LoadError::Broken(error)) => {
                error!("{error}; starting it again");
                self.0.shared.backend.put_aside(key)?;
                Ok(None)
            }
        }
    }

    /// Waits for everything queued so far to be written, for up to `limit`.
    /// `false` if some of it wasn't.
    pub fn flush(&self, limit: Duration) -> bool {
        let shared = &self.0.shared;
        let mut state = shared.lock();
        state.finishing = true;
        shared.changed.notify_all();
        let (mut state, _) = shared
            .changed
            .wait_timeout_while(state, limit, |state| !state.pending.is_empty())
            .expect("never poisoned");
        state.finishing = false;
        state.pending.is_empty()
    }
}

impl Drop for Worker {
    /// The last handle has gone: finishes writing, within `FINISH_WITHIN`.
    fn drop(&mut self) {
        let shared = &self.shared;
        let mut state = shared.lock();
        state.finishing = true;
        shared.changed.notify_all();
        let (mut state, _) = shared
            .changed
            .wait_timeout_while(state, FINISH_WITHIN, |state| !state.pending.is_empty())
            .expect("never poisoned");
        let left = state.pending.len();
        state.stop = true;
        shared.changed.notify_all();
        drop(state);
        let thread = self.thread.lock().expect("never poisoned").take();
        if left > 0 {
            // Not waiting for it: it may be stuck in the backend.
            error!("Stopping with {left} saves not written");
        } else if let Some(thread) = thread {
            let _ = thread.join();
        }
    }
}

/// The save thread: writes whatever is queued, trying again after failures.
fn run(shared: &Shared) {
    let mut retry = FIRST_RETRY;
    let mut failing = BTreeSet::new();
    loop {
        let waiting = shared
            .changed
            .wait_while(shared.lock(), |state| {
                state.pending.is_empty() && !state.stop
            })
            .expect("never poisoned");
        if waiting.stop {
            return;
        }
        drop(waiting);
        write_everything(shared, &mut failing);
        if failing.is_empty() {
            retry = FIRST_RETRY;
            continue;
        }
        // Until the time is up, or there is something new, or it is stopping.
        let state = shared.lock();
        let last_number = state.last_number;
        let (mut state, _) = shared
            .changed
            .wait_timeout_while(state, retry, |state| {
                state.last_number == last_number && !state.finishing && !state.stop
            })
            .expect("never poisoned");
        if state.stop {
            return;
        }
        // Asks for one more try, not for trying over and over.
        state.finishing = false;
        drop(state);
        retry = (retry * 2).min(LONGEST_RETRY);
    }
}

/// Tries to write everything queued, in order. `failing` is the saves that
/// wouldn't write last time: each is logged as it starts and stops failing, not
/// on every try.
fn write_everything(shared: &Shared, failing: &mut BTreeSet<SaveKey>) {
    let mut after = 0;
    loop {
        let next = shared
            .lock()
            .pending
            .iter()
            .filter(|(_, pending)| pending.number > after)
            .min_by_key(|(_, pending)| pending.number)
            .map(|(key, pending)| (key.clone(), pending.clone()));
        let Some((key, pending)) = next else {
            return;
        };
        after = pending.number;
        let result = match &pending.change {
            Change::Save(save) => shared.backend.save(&key, save),
            Change::Remove => shared.backend.remove(&key),
        };
        match result {
            Ok(()) => {
                if failing.remove(&key) {
                    info!("Saved {key:?} after all");
                }
                let mut state = shared.lock();
                // Unless a newer version has arrived meanwhile.
                if state
                    .pending
                    .get(&key)
                    .is_some_and(|queued| queued.number == pending.number)
                {
                    state.pending.remove(&key);
                }
                shared.changed.notify_all();
            }
            Err(error) => {
                if failing.insert(key.clone()) {
                    error!("Can't save {key:?}, and will keep trying: {error}");
                }
                // Sites saved after this must not be written first, as they
                // would be removed when it worked.
                if matches!(key, SaveKey::Sites(_)) {
                    return;
                }
            }
        }
    }
}

#[cfg(test)]
pub(crate) mod tests {
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
}
