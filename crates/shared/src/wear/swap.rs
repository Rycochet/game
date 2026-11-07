//! Putting something on in place of what is in its way (owner, 2026-11-07): what on
//! its layer clashes with it comes off into the inventory, so a single place can
//! take over from something on several. Refused, changing nothing, if any of that
//! is under something, won't come off (a curse, or someone else's blessing), or
//! has no room in the inventory, or if it couldn't go on anyway.

use crate::id::Id;
use crate::item::{Inventory, Item, StackLimits};
use crate::wear::{Fit, Remover, WearRefused};

/// What putting something on did: how it fits, and what came off for it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Swapped {
    pub fit: Fit,
    pub taken_off: Vec<Id>,
}

impl Inventory {
    /// Puts this carried item on (only the top of a stack), taking off what is in
    /// its way. `god` takes off even what won't come off (dev tools).
    pub fn wear_swapping(
        &mut self,
        id: Id,
        god: bool,
        limits: StackLimits,
    ) -> Result<Swapped, WearRefused> {
        if self.is_worn(id) {
            return Err(WearRefused::AlreadyWorn);
        }
        let mut after = self.clone();
        let item = after.take_top(id).ok_or(WearRefused::NoSuchItem)?;
        let swapped = after.swap_in(item, god, limits)?;
        *self = after;
        Ok(swapped)
    }

    /// The same for something not carried, such as from the ground.
    pub fn wear_new_swapping(
        &mut self,
        item: Item,
        god: bool,
        limits: StackLimits,
    ) -> Result<Swapped, WearRefused> {
        let mut after = self.clone();
        let swapped = after.swap_in(item, god, limits)?;
        *self = after;
        Ok(swapped)
    }

    /// Takes off what clashes, puts it on, and puts what came off in the inventory.
    /// Leaves `self` part done when refused: callers work on a copy.
    fn swap_in(
        &mut self,
        item: Item,
        god: bool,
        limits: StackLimits,
    ) -> Result<Swapped, WearRefused> {
        // Anything else wrong is said first: no point taking off for it.
        match self.check_wear_new(&item) {
            Ok(_) | Err(WearRefused::Clash(_)) => {}
            Err(why) => return Err(why),
        }
        let mut off = Vec::new();
        while let Some(other) = self.clash(&item) {
            if let Some(over) = self.covered_by(other) {
                return Err(WearRefused::Covered(over));
            }
            if !god && !self.may_remove(other, Remover::Wearer) {
                return Err(WearRefused::Stuck(other));
            }
            let at = self.worn.iter().position(|worn| worn.id == other);
            off.extend(at.map(|at| self.worn.remove(at)));
        }
        let fit = self.check_wear_new(&item)?;
        self.worn.push(item);
        let taken_off = off.iter().map(|item| item.id).collect();
        for item in off {
            let id = item.id;
            self.give(item, limits)
                .map_err(|_| WearRefused::NoRoom(id))?;
        }
        Ok(Swapped { fit, taken_off })
    }
}
