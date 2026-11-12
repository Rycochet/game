//! Putting things on stacks in an `Inventory`, and taking the top off one.

use crate::id::Id;
use crate::item::{Holder, Inventory, Item, MoveRefused, StackLimits};

impl Inventory {
    /// How many of `item`'s stack (carried or not), from its top, could go on the
    /// stack `onto`, which lies in a grid: refused if none could.
    pub fn stack_room(
        &self,
        item: &Item,
        onto: Id,
        limits: StackLimits,
    ) -> Result<u32, MoveRefused> {
        let (holder, placed) = self.placed(onto).ok_or(MoveRefused::NoSuchGrid)?;
        if item.holds_or_is(onto) {
            return Err(MoveRefused::IntoItself);
        }
        let plain = limits.room(&placed.item, item);
        let count = self.coin_room(item, onto, holder, limits, plain);
        if count == 0 {
            return Err(MoveRefused::NoStackRoom);
        }
        let mut part = item.clone();
        if count < part.count() {
            part = part.split_top(count);
        }
        self.may_hold(holder, &part)?;
        Ok(count)
    }

    /// Puts as much of this carried stack as fits on the stack `onto`, from its top:
    /// how many went. The rest stays where it was. Coins reaching 100 turn into
    /// the next kind (`coins.rs`).
    pub fn stack_onto(
        &mut self,
        id: Id,
        onto: Id,
        limits: StackLimits,
    ) -> Result<u32, MoveRefused> {
        let item = self.find(id).ok_or(MoveRefused::NoSuchItem)?;
        let count = self.stack_room(item, onto, limits)?;
        let holder = self.placed(onto).map(|(holder, _)| holder);
        let holder = holder.ok_or(MoveRefused::NoSuchGrid)?;
        let part = match count == item.count() {
            true => self.take(id),
            false => self.find_mut(id).map(|item| item.split_top(count)),
        };
        let part = part.ok_or(MoveRefused::NoSuchItem)?;
        self.find_mut(onto)
            .expect("checked above, and not inside what was lifted out")
            .put_on(part);
        self.carry_coins(onto, holder, limits)
            .expect("stack_room tried it on a copy");
        Ok(count)
    }

    /// Puts as much of something not carried as fits on the stack `onto`, from its
    /// top. Gives back what is left, if any.
    pub fn add_to_stack(
        &mut self,
        mut item: Item,
        onto: Id,
        limits: StackLimits,
    ) -> Result<Option<Item>, MoveRefused> {
        let count = self.stack_room(&item, onto, limits)?;
        let holder = self.placed(onto).map(|(holder, _)| holder);
        let holder = holder.ok_or(MoveRefused::NoSuchGrid)?;
        let (part, rest) = match count == item.count() {
            true => (item, None),
            false => (item.split_top(count), Some(item)),
        };
        self.find_mut(onto).expect("checked above").put_on(part);
        self.carry_coins(onto, holder, limits)
            .expect("stack_room tried it on a copy");
        Ok(rest)
    }

    /// The first stack in this grid all of `item` could go on, if any.
    pub fn first_stack_for(&self, item: &Item, holder: Holder, limits: StackLimits) -> Option<Id> {
        let grid = self.grid(holder)?;
        grid.iter()
            .map(|placed| placed.item.id)
            .find(|&onto| self.stack_room(item, onto, limits) == Ok(item.count()))
    }

    /// Takes the top item off a carried stack, or the item itself if it is alone.
    pub(crate) fn take_top(&mut self, id: Id) -> Option<Item> {
        match self.find(id)?.count() {
            1 => self.take(id),
            _ => self.find_mut(id)?.take_top(),
        }
    }

    /// The carried item with this id, to change.
    pub(super) fn find_mut(&mut self, id: Id) -> Option<&mut Item> {
        if self.in_bin(id) {
            return self.bin.as_mut();
        }
        if let Some(index) = self.worn.iter().position(|item| item.id == id) {
            return Some(&mut self.worn[index]);
        }
        let (holder, _) = self.placed(id)?;
        let grid = self.grid_mut(holder)?;
        grid.iter_mut()
            .map(|placed| &mut placed.item)
            .find(|item| item.id == id)
    }
}
