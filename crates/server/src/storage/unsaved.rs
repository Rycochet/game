//! `Unsaved`: what has changed since it was saved, and `now`.

use std::collections::BTreeSet;
use std::time::{SystemTime, UNIX_EPOCH};

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

    /// No longer marked.
    pub fn unmark(&mut self, key: &K) {
        self.0.remove(key);
    }

    /// Everything marked, forgetting it.
    pub fn take(&mut self) -> BTreeSet<K> {
        std::mem::take(&mut self.0)
    }
}
