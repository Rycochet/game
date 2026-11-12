//! Coins turning into the next kind as they join a stack (owner, 2026-11-12):
//! whenever a copper or silver stack would reach 100, each 100 become one of the
//! next kind, put on a stack of that kind in the same grid or else in its first
//! free place, carrying on upward. What could not then find room stays where it
//! came from. The same rule on both sides, so the client's guess is the server's
//! answer: a new coin takes the id of the first of the hundred it is made from.

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
                let mut trial = self.clone();
                trial.join_coins(item, onto, holder, count, limits).is_ok()
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
    ) -> Result<(), MoveRefused> {
        let mut part = item.clone();
        if count < part.count() {
            part = part.split_top(count);
        }
        // Something carried leaves the place it lay in.
        if self.find(item.id).is_some() {
            match count == item.count() {
                true => drop(self.take(item.id)),
                false => drop(self.find_mut(item.id).map(|item| item.split_top(count))),
            }
        }
        self.find_mut(onto)
            .ok_or(MoveRefused::NoSuchItem)?
            .put_on(part);
        self.carry_coins(onto, holder, limits)
    }

    /// Turns each 100 coins in the stack `onto` (lying in `holder`) into one of
    /// the next kind, leaving the rest from the bottom, its id the stack's while
    /// any are left. Refused if what is left is more than a stack may hold, or a
    /// new coin has no room: then it is half done, so it is tried on a copy first.
    pub(super) fn carry_coins(
        &mut self,
        onto: Id,
        holder: Holder,
        limits: StackLimits,
    ) -> Result<(), MoveRefused> {
        let stack = self.find(onto).ok_or(MoveRefused::NoSuchItem)?;
        let Some(next) = stack.kind.next_coin() else {
            return Ok(());
        };
        let keep = stack.count() % COINS_PER_NEXT;
        if keep > limits.most_of(stack) {
            return Err(MoveRefused::NoStackRoom);
        }
        if stack.count() < COINS_PER_NEXT {
            return Ok(());
        }
        let turned = match keep {
            0 => {
                let mut bottom = self.take(onto).ok_or(MoveRefused::NoSuchItem)?;
                let above = std::mem::take(&mut bottom.above);
                std::iter::once(bottom).chain(above).collect::<Vec<_>>()
            }
            _ => {
                let stack = self.find_mut(onto).ok_or(MoveRefused::NoSuchItem)?;
                stack.above.split_off(keep as usize - 1)
            }
        };
        for hundred in turned.chunks(COINS_PER_NEXT as usize) {
            let first = &hundred[0];
            let coin = Item {
                made_at: first.made_at,
                ..Item::new(first.id, next)
            };
            self.put_coin(coin, holder, limits)?;
        }
        Ok(())
    }

    /// Puts a new coin in this grid: on a stack of its kind, or else in the
    /// first place it fits.
    fn put_coin(
        &mut self,
        coin: Item,
        holder: Holder,
        limits: StackLimits,
    ) -> Result<(), MoveRefused> {
        if let Some(onto) = self.first_stack_for(&coin, holder, limits) {
            return self.add_to_stack(coin, onto, limits).map(drop);
        }
        let (x, y, rotation) = self
            .first_place_for_new(&coin, holder, Rotation::None)
            .ok_or(MoveRefused::NoRoom)?;
        self.add(coin, holder, x, y, rotation)
    }
}
