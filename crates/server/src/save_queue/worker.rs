//! The save thread's loop: writing what is queued, and trying again after failures.

use std::collections::BTreeSet;

use bevy::prelude::*;

use super::{Change, FIRST_RETRY, LONGEST_RETRY, Shared};
use crate::saves::SaveKey;

/// The save thread: writes whatever is queued, trying again after failures.
pub(super) fn run(shared: &Shared) {
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
