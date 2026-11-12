//! Coins turning into the next kind as they join a stack (owner, 2026-11-12):
//! whenever a copper or silver stack would reach 100, each 100 become one of the
//! next kind, put on a stack of that kind in the same grid or else in its first
//! free place, carrying on upward. What could not then find room stays where it
//! came from. Coins are counted (`ItemDef::counted`), so this is sums on counts;
//! a new stack's id comes from `new_id`: the server's own, or a stand-in in the
//! client's guess, which the server's answer replaces.

use crate::id::Id;
use crate::item::{COINS_PER_NEXT, Holder, Inventory, Item, MoveRefused, Rotation, StackLimits};

impl Inventory {
    /// How many of `item`'s stack, from its top, could go on the stack `onto`
    /// (lying in `holder`), hundreds of coins turning into the next coin: for
    /// anything else `plain`, the number that fit by the usual rule.
    pub(super) fn coin_room(
        &self,
        item: &Item,
        onto: Id,
        holder: Holder,
        limits: StackLimits,
        plain: u32,
    ) -> u32 {
        let Some(stack) = self.find(onto) else {
            return plain;
        };
        let joins = stack.kind.next_coin().is_some()
            && item.kind == stack.kind
            && stack.is_even()
            && item.members().all(|one| one.looks_like(stack));
        if !joins {
            return plain;
        }
        // Most often all of it goes, the first tried. Short of 100 nothing turns,
        // and the plain rule holds.
        let before = stack.count();
        (1..=item.count())
            .rev()
            .find(|&count| {
                if before + count < COINS_PER_NEXT {
                    return count <= plain;
                }
                // Stand-ins from the top down, apart from the client's guesses'.
                let mut next = u64::MAX;
                let mut stand_in = || {
                    next -= 1;
                    Id::stand_in(next)
                };
                let mut trial = self.clone();
                trial
                    .join_coins(item, onto, holder, count, limits, &mut stand_in)
                    .is_ok()
            })
            .unwrap_or(0)
    }

    /// Puts the top `count` of `item` (carried or not) on the coin stack `onto`
    /// and turns its hundreds into the next coin. Refused half done, so only for
    /// a copy, to try it.
    fn join_coins(
        &mut self,
        item: &Item,
        onto: Id,
        holder: Holder,
        count: u32,
        limits: StackLimits,
        new_id: &mut impl FnMut() -> Id,
    ) -> Result<(), MoveRefused> {
        // Something carried leaves the place it lay in.
        if self.find(item.id).is_some() {
            match count == item.count() {
                true => drop(self.take(item.id)),
                false => self.find_mut(item.id).ok_or(MoveRefused::NoSuchItem)?.count -= count,
            }
        }
        self.find_mut(onto).ok_or(MoveRefused::NoSuchItem)?.count += count;
        self.carry_coins(onto, holder, limits, new_id)
    }

    /// Turns each 100 coins in the stack `onto` (lying in `holder`) into one of
    /// the next kind, leaving the rest, with the stack's id, while any are left.
    /// Refused if what is left is more than a stack may hold, or a new coin has
    /// no room: then it is half done, so it is tried on a copy first.
    pub(super) fn carry_coins(
        &mut self,
        onto: Id,
        holder: Holder,
        limits: StackLimits,
        new_id: &mut impl FnMut() -> Id,
    ) -> Result<(), MoveRefused> {
        let stack = self.find(onto).ok_or(MoveRefused::NoSuchItem)?;
        let Some(next) = stack.kind.next_coin() else {
            return Ok(());
        };
        let all = stack.count();
        let keep = all % COINS_PER_NEXT;
        if keep > limits.most_of(stack) {
            return Err(MoveRefused::NoStackRoom);
        }
        if all < COINS_PER_NEXT {
            return Ok(());
        }
        let made_at = stack.made_at;
        match keep {
            0 => drop(self.take(onto)),
            _ => self.find_mut(onto).ok_or(MoveRefused::NoSuchItem)?.count = keep,
        }
        let coins = Item {
            made_at,
            count: all / COINS_PER_NEXT,
            ..Item::new(new_id(), next)
        };
        self.put_coins(coins, holder, limits, new_id)
    }

    /// Puts new coins in this grid: on the stacks of their kind with room, in
    /// order, and the rest in the first places they fit, as many on each as a
    /// stack may hold.
    fn put_coins(
        &mut self,
        mut coins: Item,
        holder: Holder,
        limits: StackLimits,
        new_id: &mut impl FnMut() -> Id,
    ) -> Result<(), MoveRefused> {
        loop {
            let grid = self.grid(holder).ok_or(MoveRefused::NoSuchGrid)?;
            let ids: Vec<Id> = grid.iter().map(|placed| placed.item.id).collect();
            let onto = ids
                .into_iter()
                .find(|&onto| self.stack_room(&coins, onto, limits).is_ok());
            if let Some(onto) = onto {
                match self.add_to_stack(coins, onto, limits, new_id)? {
                    Some(rest) => coins = rest,
                    None => return Ok(()),
                }
                continue;
            }
            let most = limits.most_of(&coins);
            let rest = (coins.count > most).then(|| coins.split_top(coins.count - most, new_id));
            let (x, y, rotation) = self
                .first_place_for_new(&coins, holder, Rotation::None)
                .ok_or(MoveRefused::NoRoom)?;
            self.add(coins, holder, x, y, rotation)?;
            match rest {
                Some(rest) => coins = rest,
                None => return Ok(()),
            }
        }
    }
}
