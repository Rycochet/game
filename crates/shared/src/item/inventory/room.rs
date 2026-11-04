//! Finding room for something in an `Inventory`, and giving new items.

use crate::id::Id;
use crate::item::{Holder, Inventory, Item, Placed, Rotation, Shape, StackLimits};

impl Inventory {
    /// The first place in this grid where an item of this shape has room,
    /// starting with how it is turned now: rows from the top, then the next
    /// quarter turn. `moving` is lifted out, if it is carried here already.
    pub(super) fn first_room(
        &self,
        holder: Holder,
        shape: Shape,
        turned_from: Rotation,
        moving: Option<Id>,
    ) -> Option<(u8, u8, Rotation)> {
        let (width, height) = self.grid_size(holder)?;
        let mut rotation = turned_from;
        for _ in Rotation::ALL {
            let turned = shape.turned(rotation);
            for y in 0..height {
                for x in 0..width {
                    if self.has_room(holder, turned, x, y, moving) {
                        return Some((x, y, rotation));
                    }
                }
            }
            rotation = rotation.next();
        }
        None
    }

    /// Where this carried item would go if dropped on this grid without saying
    /// where (on a closed bag): the first place it can.
    pub fn first_place_for(
        &self,
        id: Id,
        holder: Holder,
        turned_from: Rotation,
    ) -> Option<(u8, u8, Rotation)> {
        let item = self.find(id)?;
        self.may_hold(holder, item).ok()?;
        self.first_room(holder, item.shape(), turned_from, Some(id))
    }

    /// Puts a new item in the inventory: on the first stack it can all go on, or else
    /// in the first place it fits. Gives it back if there is no room.
    #[allow(clippy::result_large_err)] // Handing it back whole is the point, and rare.
    pub fn give(&mut self, item: Item, limits: StackLimits) -> Result<(), Item> {
        if let Some(onto) = self.first_stack_for(&item, Holder::Inventory, limits) {
            self.add_to_stack(item, onto, limits)
                .expect("first_stack_for found room for all of it");
            return Ok(());
        }
        let shape = item.shape();
        match self.first_room(Holder::Inventory, shape, Rotation::None, None) {
            Some((x, y, rotation)) => {
                self.items.push(Placed {
                    item,
                    x,
                    y,
                    rotation,
                });
                Ok(())
            }
            None => Err(item),
        }
    }
}
