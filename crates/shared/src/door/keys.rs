//! Keys: whether someone carries the one for a lock.

use crate::id::Id;
use crate::item::{Inventory, Item};
use crate::item_form::ItemForm;

/// Whether `item`, or anything inside it or stacked on it, is the key to `lock`.
fn is_or_holds_key(item: &Item, lock: Id) -> bool {
    item.form == Some(ItemForm::Key(lock))
        || item
            .contents
            .iter()
            .any(|placed| is_or_holds_key(&placed.item, lock))
        || item.above.iter().any(|above| is_or_holds_key(above, lock))
}

/// Whether the key to `lock` is carried, in bags too, or worn (not binned).
pub fn carries_key(inventory: &Inventory, lock: Id) -> bool {
    let carried = inventory.items.iter().map(|placed| &placed.item);
    carried
        .chain(&inventory.worn)
        .any(|item| is_or_holds_key(item, lock))
}
