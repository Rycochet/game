//! Forgetting sites, so they are made anew, and noticing that it happened.

use super::ServerWorld;
use crate::storage::SiteId;
use std::collections::BTreeSet;

/// Which sites have been forgotten.
#[derive(Default)]
pub(crate) struct Forgotten {
    all: bool,
    sites: BTreeSet<SiteId>,
}

impl Forgotten {
    pub(crate) fn is_empty(&self) -> bool {
        !self.all && self.sites.is_empty()
    }

    pub(crate) fn includes(&self, id: SiteId) -> bool {
        self.all || self.sites.contains(&id)
    }
}

impl ServerWorld {
    /// Forgets a site: it is made anew when next entered, with its generator's
    /// current version and no players' changes.
    pub(crate) fn forget_site(&mut self, id: SiteId) {
        self.sites.remove(&id);
        self.unreachable_sites.remove(&id);
        self.unsaved_sites.unmark(&id);
        self.storage.remove_site(id);
        self.populations.forget_site(id);
        self.forgotten.sites.insert(id);
        if let SiteId::Poi(chunk) = id {
            self.wilderness.forget(Some(chunk));
        }
    }

    /// Forgets every site, as `forget_site`.
    pub(crate) fn forget_sites(&mut self) {
        self.sites.clear();
        self.unreachable_sites.clear();
        self.unsaved_sites.take();
        self.storage.remove_sites();
        self.populations.forget_all();
        self.forgotten.all = true;
        self.wilderness.forget(None);
    }

    /// The sites forgotten since this was last asked.
    pub(crate) fn take_forgotten(&mut self) -> Forgotten {
        std::mem::take(&mut self.forgotten)
    }
}
