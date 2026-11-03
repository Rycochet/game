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
pub(crate) mod tests;
