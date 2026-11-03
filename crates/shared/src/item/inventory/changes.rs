//! Changing an `Inventory`: adding, moving and throwing things away.

use crate::id::Id;
use crate::item::{Holder, Inventory, Item, MoveRefused, Placed, Rotation};

impl Inventory {
    /// Puts something not carried here, if it can go.
    pub fn add(
        &mut self,
        item: Item,
        holder: Holder,
        x: u8,
        y: u8,
        rotation: Rotation,
    ) -> Result<(), MoveRefused> {
        self.check_add(&item, holder, x, y, rotation)?;
        self.grid_mut(holder).expect("checked above").push(Placed {
            item,
            x,
            y,
            rotation,
        });
        Ok(())
    }

    /// The first place something not carried fits in this container, turning if it
    /// must, as `first_place_for`.
    pub fn first_place_for_new(
        &self,
        item: &Item,
        holder: Holder,
        turned_from: Rotation,
    ) -> Option<(u8, u8, Rotation)> {
        self.may_hold(holder, item).ok()?;
        self.first_room(holder, item.shape(), turned_from, None)
    }

    /// Moves a carried item here, if it can go.
    pub fn move_item(
        &mut self,
        id: Id,
        holder: Holder,
        x: u8,
        y: u8,
        rotation: Rotation,
    ) -> Result<(), MoveRefused> {
        self.check_move(id, holder, x, y, rotation)?;
        let item = self.take(id).ok_or(MoveRefused::NoSuchItem)?;
        self.grid_mut(holder)
            .expect("checked above, and not inside what was lifted out")
            .push(Placed {
                item,
                x,
                y,
                rotation,
            });
        Ok(())
    }

    /// Throws a carried item away, with everything in it, destroying what was thrown
    /// away before. `false` if it isn't carried, is worn under something, or is
    /// there already.
    pub fn to_bin(&mut self, id: Id) -> bool {
        if self.in_bin(id) || self.covered_by(id).is_some() {
            return false;
        }
        let Some(item) = self.take(id) else {
            return false;
        };
        self.bin = Some(item);
        true
    }

    /// Throws away something not carried, destroying what was thrown away before.
    pub fn to_bin_new(&mut self, item: Item) {
        self.bin = Some(item);
    }

    /// Whether this is what was last thrown away.
    pub fn in_bin(&self, id: Id) -> bool {
        self.bin.as_ref().is_some_and(|item| item.id == id)
    }

    /// Takes an item out, wherever it is.
    pub(crate) fn take(&mut self, id: Id) -> Option<Item> {
        fn search(placed: &mut Vec<Placed>, id: Id) -> Option<Item> {
            if let Some(index) = placed.iter().position(|placed| placed.item.id == id) {
                return Some(placed.remove(index).item);
            }
            placed
                .iter_mut()
                .find_map(|placed| search(&mut placed.item.contents, id))
        }
        if self.in_bin(id) {
            return self.bin.take();
        }
        if let Some(index) = self.worn.iter().position(|item| item.id == id) {
            return Some(self.worn.remove(index));
        }
        search(&mut self.pockets, id).or_else(|| {
            self.worn
                .iter_mut()
                .find_map(|item| search(&mut item.contents, id))
        })
    }

    pub(super) fn grid_mut(&mut self, holder: Holder) -> Option<&mut Vec<Placed>> {
        fn search(placed: &mut [Placed], id: Id) -> Option<&mut Vec<Placed>> {
            placed.iter_mut().find_map(|placed| {
                if placed.item.id == id {
                    Some(&mut placed.item.contents)
                } else {
                    search(&mut placed.item.contents, id)
                }
            })
        }
        match holder {
            Holder::Pockets => Some(&mut self.pockets),
            Holder::Inside(id) => search(&mut self.pockets, id).or_else(|| {
                self.worn.iter_mut().find_map(|item| {
                    if item.id == id {
                        Some(&mut item.contents)
                    } else {
                        search(&mut item.contents, id)
                    }
                })
            }),
        }
    }
}
