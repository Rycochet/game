//! Loading, saving and removing sites.

use bevy::prelude::*;

use super::{SiteId, SiteSave, Storage};
use crate::saves::{Save, SaveKey};

impl Storage {
    /// Whether a save made under `forgotten_to` still counts.
    pub(super) fn not_forgotten(&self, forgotten_to: u32, what: &str) -> bool {
        let kept = forgotten_to >= self.forget_before;
        if !kept {
            info!("Forgetting {what}: saved before {}", self.forget_before);
        }
        kept
    }

    /// A saved site, if it has been entered before. A broken one is put aside
    /// (and logged), so the site starts again; one from before `forget_before` is
    /// ignored, and saved over. An error if it couldn't be reached: it may be
    /// fine, so it mustn't be started again.
    pub fn load_site(&self, site: SiteId) -> Result<Option<SiteSave>, String> {
        let Some(queue) = &self.queue else {
            return Ok(None);
        };
        Ok(match queue.load(&SaveKey::Site(self.world, site))? {
            Some(Save::Site(save)) => self
                .not_forgotten(save.forgotten_to, &format!("{site:?}"))
                .then_some(save),
            _ => None,
        })
    }

    /// Deletes a site's save, if it has one.
    pub fn remove_site(&self, site: SiteId) {
        if let Some(queue) = &self.queue {
            queue.remove(SaveKey::Site(self.world, site));
        }
    }

    /// Deletes every site's save.
    pub fn remove_sites(&self) {
        if let Some(queue) = &self.queue {
            queue.remove(SaveKey::Sites(self.world));
        }
    }

    pub fn save_site(&self, site: SiteId, save: &SiteSave) {
        if let Some(queue) = &self.queue {
            let forgotten_to = self.forget_before;
            queue.save(
                SaveKey::Site(self.world, site),
                Save::Site(SiteSave {
                    forgotten_to,
                    ..save.clone()
                }),
            );
        }
    }
}
