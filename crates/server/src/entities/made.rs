//! Making a mob from how it was rolled, and telling ones made by developer
//! tools from a level's own.

use shared::creature::Size;
use shared::id::Id;
use shared::item::Inventory;
use shared::mob::{Memory, Spawn, scent_strength};
use shared::sheet::Sheet;
use shared::spot::Spot;

use super::{Doing, Mob};
use crate::storage::SiteId;

/// Numbers from here on are mobs made by developer tools (Create Mob): no
/// level's own mob has one, and nothing about them is kept.
pub(super) const DEV_MOB_NUMBERS: u32 = 1 << 31;
/// A site's quest boss: kept like a level's own (`dead` while it is down).
pub(crate) const BOSS_NUMBER: u32 = DEV_MOB_NUMBERS - 1;
/// Someone a quester escorts (`escortee`): never kept.
pub(super) const ESCORT_NUMBER: u32 = BOSS_NUMBER - 1;
/// Numbers from here up to the escortee's are extras a quest's "Slay" brings
/// (`quest_mobs`): never kept.
pub(super) const EXTRA_NUMBERS: u32 = 1 << 30;

impl Mob {
    /// A mob as rolled (`spawn`), carrying `carrying`, belonging on level
    /// `home_level`: at rest at home, remembering nothing.
    pub(super) fn new(id: Id, spawn: &Spawn, carrying: &Inventory, home_level: u32) -> Self {
        let sheet = Sheet {
            attributes: spawn.attributes,
            gender: spawn.gender,
            ..Sheet::new(spawn.kind)
        };
        // Dressed, as players are: mobs don't change clothes, so once.
        let bonuses = sheet.totals(&carrying.worn, 0);
        Mob {
            id,
            kind: spawn.kind,
            gender: spawn.gender,
            alignment: spawn.alignment,
            appearance: spawn.appearance.clone(),
            position: spawn.position,
            spot: Spot::middle_of(spawn.position),
            // Each its own way to begin with.
            heading: shared::facing::Heading(id.0.as_u128() as u16),
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
            attributes: bonuses.attributes(&spawn.attributes),
            bonuses,
            vitals: Default::default(),
            wielded: shared::combat::Wielded::of(carrying),
            fight: Default::default(),
            harmed_by: Default::default(),
            pacing: Default::default(),
            still: false,
            kept_in: None,
            escort: None,
        }
    }

    /// The most Health and Mana it can have, with its bonuses.
    pub(super) fn limits(&self) -> shared::vitals::Limits {
        (self.vitals).limits_with(&self.attributes, &self.bonuses, 0, None)
    }

    /// The site whose save keeps it, for a mob on a level of `here`: its own
    /// site's, if it is a site's mob about in the wilderness.
    pub(super) fn kept_with(&self, here: SiteId) -> SiteId {
        self.kept_in.unwrap_or(here)
    }

    /// Puts its body at `spot`, and its thinking on the cell under it.
    pub(crate) fn place(&mut self, spot: Spot) {
        self.spot = spot;
        self.position = spot.cell();
    }

    /// Puts it at a cell's middle.
    pub(crate) fn put(&mut self, cell: (i64, i64)) {
        self.place(Spot::middle_of(cell));
    }

    /// How big it is, by its kind: its circle on levels.
    pub(crate) fn size(&self) -> Size {
        self.kind.def().size
    }

    /// Whether developer tools made it: it is never kept.
    pub(super) fn is_dev_made(&self) -> bool {
        self.number >= DEV_MOB_NUMBERS
    }

    /// Whether it is its site's quest boss.
    pub(crate) fn is_boss(&self) -> bool {
        self.number == BOSS_NUMBER
    }

    /// Whether its site's save keeps it: a level's own or the boss, not one
    /// developer tools made or a quest's extra.
    pub(crate) fn is_kept(&self) -> bool {
        self.number < EXTRA_NUMBERS || self.is_boss()
    }
}
