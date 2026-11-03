//! The rules a move or an addition is checked by.

use crate::id::Id;
use crate::item::item::weight_of;
use crate::item::{Holder, Inventory, Item, MAX_DEPTH, MoveRefused, Placed, Rotation, Shape};

impl Inventory {
    /// Whether `shape` fits at `(x, y)` in this grid, not counting `moving`,
    /// which is lifted out.
    pub(super) fn has_room(
        &self,
        holder: Holder,
        shape: Shape,
        x: u8,
        y: u8,
        moving: Option<Id>,
    ) -> bool {
        let (Some(grid), Some((width, height))) = (self.grid(holder), self.grid_size(holder))
        else {
            return false;
        };
        let inside = x as u32 + shape.width as u32 <= width as u32
            && y as u32 + shape.height as u32 <= height as u32;
        inside
            && !grid
                .iter()
                .filter(|placed| Some(placed.item.id) != moving)
                .flat_map(Placed::cells)
                .any(|(cell_x, cell_y)| {
                    cell_x >= x && cell_y >= y && shape.has(cell_x - x, cell_y - y)
                })
    }

    /// Whether this carried item may go in this grid, wherever in it: all but
    /// the room.
    pub(super) fn may_hold(&self, holder: Holder, item: &Item) -> Result<(), MoveRefused> {
        let Holder::Inside(target) = holder else {
            return Ok(());
        };
        if item.holds_or_is(target) {
            return Err(MoveRefused::IntoItself);
        }
        let target_item = self.find(target).ok_or(MoveRefused::NoSuchGrid)?;
        if target_item.kind.def().container.is_none() {
            return Err(MoveRefused::NoSuchGrid);
        }
        if !item.shape().smaller_than(target_item.shape()) {
            return Err(MoveRefused::TooBig);
        }
        let mut chain = self
            .containers_around(target)
            .ok_or(MoveRefused::NoSuchGrid)?;
        chain.push(target);
        if chain.len() as u32 + item.grids_deep() > MAX_DEPTH {
            return Err(MoveRefused::TooDeep);
        }
        // From the inside out: each container it would end up in, with what
        // would lie directly in that one on the way to the item. A container it
        // is in already gains nothing.
        let weight = item.total_weight();
        let mut lying_in_it = weight;
        for &container in chain.iter().rev() {
            let holding = self.find(container).ok_or(MoveRefused::NoSuchGrid)?;
            let def = holding
                .kind
                .def()
                .container
                .ok_or(MoveRefused::NoSuchGrid)?;
            let gained = if holding.holds_or_is(item.id) {
                0
            } else {
                weight
            };
            let total = weight_of(&holding.contents) + gained;
            if def.max_total.is_some_and(|most| total > most)
                || def.max_each.is_some_and(|most| lying_in_it > most)
            {
                return Err(MoveRefused::TooHeavy);
            }
            lying_in_it = holding.total_weight() + gained;
        }
        Ok(())
    }

    /// Whether this carried item can go here, turned this way.
    pub fn check_move(
        &self,
        id: Id,
        holder: Holder,
        x: u8,
        y: u8,
        rotation: Rotation,
    ) -> Result<(), MoveRefused> {
        let item = self.find(id).ok_or(MoveRefused::NoSuchItem)?;
        if self.covered_by(id).is_some() {
            return Err(MoveRefused::Covered);
        }
        self.may_hold(holder, item)?;
        let shape = item.shape().turned(rotation);
        if self.grid(holder).is_none() {
            return Err(MoveRefused::NoSuchGrid);
        }
        if self.has_room(holder, shape, x, y, Some(id)) {
            Ok(())
        } else {
            Err(MoveRefused::NoRoom)
        }
    }

    /// Whether something not carried could be put here.
    pub fn check_add(
        &self,
        item: &Item,
        holder: Holder,
        x: u8,
        y: u8,
        rotation: Rotation,
    ) -> Result<(), MoveRefused> {
        self.may_hold(holder, item)?;
        if self.grid(holder).is_none() {
            return Err(MoveRefused::NoSuchGrid);
        }
        let shape = item.shape().turned(rotation);
        if self.has_room(holder, shape, x, y, None) {
            Ok(())
        } else {
            Err(MoveRefused::NoRoom)
        }
    }
}
