//! Finding an item by id in what a world has loaded: its sites' ground, shops and
//! mobs (players carry their own: `Client::inventory`).

use shared::id::Id;
use shared::item::Item;

use super::ServerWorld;
use crate::storage::LevelSave;

impl ServerWorld {
    /// The item with this id, if it is anywhere in a site loaded now (the
    /// wilderness included): on the ground, for sale or with a mob.
    pub(crate) fn find_item(&self, id: Id) -> Option<&Item> {
        let levels = self.sites.values().flat_map(|site| site.levels.values());
        levels.into_iter().find_map(|level| on_level(level, id))
    }
}

fn on_level(level: &LevelSave, id: Id) -> Option<&Item> {
    let lying = level.ground.values().flatten();
    let for_sale = level.buildings.values().flat_map(|saved| &saved.stock);
    let carried = level.records.values().flat_map(|record| &record.carried);
    let mut loose = lying.chain(for_sale).chain(carried);
    loose.find_map(|item| item.find_within(id)).or_else(|| {
        let geared = level
            .records
            .values()
            .filter_map(|record| record.gear.as_ref());
        geared.into_iter().find_map(|gear| gear.find_within(id))
    })
}
