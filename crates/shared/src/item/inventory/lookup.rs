//! Finding things in an `Inventory`: items, grids and what holds what.

use crate::id::Id;
use crate::item::{Holder, Inventory, Item, Placed};

impl Inventory {
    /// The item with this id, wherever it is.
    pub fn find(&self, id: Id) -> Option<&Item> {
        fn search(placed: &[Placed], id: Id) -> Option<&Item> {
            placed.iter().find_map(|placed| {
                if placed.item.id == id {
                    Some(&placed.item)
                } else {
                    search(&placed.item.contents, id)
                }
            })
        }
        let carried = search(&self.items, id).or_else(|| {
            self.worn.iter().find_map(|item| {
                if item.id == id {
                    Some(item)
                } else {
                    search(&item.contents, id)
                }
            })
        });
        carried.or_else(|| self.bin.as_ref().filter(|item| item.id == id))
    }

    /// What is in this grid: `None` if there is no such grid.
    pub fn grid(&self, holder: Holder) -> Option<&[Placed]> {
        match holder {
            Holder::Inventory => Some(&self.items),
            Holder::Inside(id) => {
                let item = self.find(id)?;
                item.kind.def().container?;
                Some(&item.contents)
            }
        }
    }

    /// How wide and tall this grid is.
    pub fn grid_size(&self, holder: Holder) -> Option<(u8, u8)> {
        match holder {
            Holder::Inventory => Some(self.size),
            Holder::Inside(id) => {
                let container = self.find(id)?.kind.def().container?;
                Some((container.width, container.height))
            }
        }
    }

    /// The containers leading to `id` from the inventory, outermost first (not
    /// `id` itself): `None` if it isn't here.
    pub(super) fn containers_around(&self, id: Id) -> Option<Vec<Id>> {
        fn search(placed: &[Placed], id: Id, around: &mut Vec<Id>) -> bool {
            placed.iter().any(|placed| {
                if placed.item.id == id {
                    return true;
                }
                around.push(placed.item.id);
                let found = search(&placed.item.contents, id, around);
                if !found {
                    around.pop();
                }
                found
            })
        }
        let mut around = Vec::new();
        if search(&self.items, id, &mut around) {
            return Some(around);
        }
        for item in &self.worn {
            if item.id == id {
                return Some(Vec::new());
            }
            let mut around = vec![item.id];
            if search(&item.contents, id, &mut around) {
                return Some(around);
            }
        }
        None
    }

    /// Which grid an item is in, and how it lies there. None if it isn't in a
    /// grid, as when it is worn.
    pub fn placed(&self, id: Id) -> Option<(Holder, &Placed)> {
        if self.is_worn(id) || self.in_bin(id) {
            return None;
        }
        let holder = match self.containers_around(id)?.last() {
            Some(&container) => Holder::Inside(container),
            None => Holder::Inventory,
        };
        let placed = self
            .grid(holder)?
            .iter()
            .find(|placed| placed.item.id == id)?;
        Some((holder, placed))
    }
}
