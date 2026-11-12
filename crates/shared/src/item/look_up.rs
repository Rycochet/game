//! Finding one item by id anywhere inside another or in an inventory, and the
//! copy of it told to someone who asked after it by id (`crate::item_text`).

use crate::id::Id;
use crate::item::{Inventory, Item};

impl Item {
    /// This item or one inside it, at any depth, or stacked on it: the one with
    /// this id.
    pub fn find_within(&self, id: Id) -> Option<&Item> {
        if self.id == id {
            return Some(self);
        }
        let inside = self.contents.iter().map(|placed| &placed.item);
        inside
            .chain(&self.above)
            .find_map(|item| item.find_within(id))
    }

    /// It alone: nothing it holds or has stacked on it, which says nothing of
    /// those to whoever asked after it by its id.
    pub fn alone(&self) -> Item {
        Item {
            contents: Vec::new(),
            above: Vec::new(),
            count: 1,
            ..self.clone()
        }
    }
}

impl Inventory {
    /// The item with this id anywhere here, stacked ones included: carried, worn
    /// or in the bin.
    pub fn find_within(&self, id: Id) -> Option<&Item> {
        let carried = self.items.iter().map(|placed| &placed.item);
        carried
            .chain(&self.worn)
            .chain(&self.bin)
            .find_map(|item| item.find_within(id))
    }
}
