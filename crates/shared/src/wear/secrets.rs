//! Finding out what is secret about something worn, and hiding it from clients.

use crate::id::Id;
use crate::item::{Inventory, Made};
use crate::wear::made_for_fingers;

impl Inventory {
    /// Finds out that this worn thing is cursed or blessed.
    pub fn favour_found_out(&mut self, id: Id) {
        if let Some(item) = self.worn.iter_mut().find(|item| item.id == id) {
            item.known.favour = true;
        }
    }

    /// Whether putting this on just now was only possible by its magic, which gives
    /// it away. A thing that would fit anyway keeps its secret.
    pub fn magic_found_out(&mut self, id: Id) -> bool {
        let body = self.creature.def();
        let Some(item) = self.worn.iter_mut().find(|item| item.id == id) else {
            return false;
        };
        let Some(wear) = item.kind.def().wear else {
            return false;
        };
        if !item.is_magical() {
            return false;
        }
        let without = Made {
            magical: false,
            ..item.made
        };
        let plain = without.fit(wear.adjustable, body.size, item.quality);
        let with = item.made.fit(wear.adjustable, body.size, item.quality);
        let whole_fingers = made_for_fingers(item, &wear);
        let needed = (plain != with) || (whole_fingers && item.made.fingers != body.fingers());
        if needed {
            item.known.magic = true;
        }
        needed
    }

    /// As a client may see it: see `Item::hide_secrets`.
    pub fn hide_secrets(&mut self) {
        let placed = self.items.iter_mut().map(|placed| &mut placed.item);
        for item in placed.chain(&mut self.worn).chain(&mut self.bin) {
            item.hide_secrets();
        }
    }
}
