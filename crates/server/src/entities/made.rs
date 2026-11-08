//! Making a mob from how it was rolled, and telling ones made by developer
//! tools from a level's own.

use shared::id::Id;
use shared::item::Inventory;
use shared::mob::{Memory, Spawn, scent_strength};
use shared::sheet::Sheet;

use super::{Doing, Mob};
use crate::storage::SiteId;

/// Numbers from here on are mobs made by developer tools (Create Mob): no
/// level's own mob has one, and nothing about them is kept.
pub(super) const DEV_MOB_NUMBERS: u32 = 1 << 31;

impl Mob {
    /// A mob as rolled (`spawn`), carrying `carrying`, belonging on level
    /// `home_level`: at rest at home, remembering nothing.
    pub(super) fn new(id: Id, spawn: &Spawn, carrying: &Inventory, home_level: u32) -> Self {
        let sheet = Sheet {
            attributes: spawn.attributes,
            ..Sheet::new(spawn.kind)
        };
        Mob {
            id,
            kind: spawn.kind,
            appearance: spawn.appearance.clone(),
            position: spawn.position,
            home: spawn.position,
            doing: Doing::Idle,
            target: None,
            number: spawn.number,
            memory: Memory::default(),
            home_level,
            way_back: Vec::new(),
            energy: Default::default(),
            speed: sheet.speed(carrying),
            shown: carrying.shown(),
            scent: scent_strength(carrying),
            attributes: spawn.attributes,
            vitals: Default::default(),
            wielded: shared::combat::Wielded::of(carrying),
            fight: Default::default(),
            harmed_by: Default::default(),
            pacing: Default::default(),
            still: false,
            kept_in: None,
        }
    }

    /// The site whose save keeps it, for a mob on a level of `here`: its own
    /// site's, if it is a site's mob about in the wilderness.
    pub(super) fn kept_with(&self, here: SiteId) -> SiteId {
        self.kept_in.unwrap_or(here)
    }

    /// Whether developer tools made it: it is never kept.
    pub(super) fn is_dev_made(&self) -> bool {
        self.number >= DEV_MOB_NUMBERS
    }
}
