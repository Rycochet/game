//! Putting things on and taking them off, handing them over, and changing body.

use crate::creature::CreatureKind;
use crate::id::Id;
use crate::item::{Favour, Inventory, Item, StackLimits};
use crate::wear::{Fit, Remover, WearRefused};

impl Inventory {
    /// Puts this carried item on, if it can go.
    pub fn wear(&mut self, id: Id) -> Result<Fit, WearRefused> {
        let fit = self.check_wear(id)?;
        // Only the top of a stack: the rest stays where it was.
        let item = self.take_top(id).ok_or(WearRefused::NoSuchItem)?;
        self.worn.push(item);
        Ok(fit)
    }

    /// Puts on something not carried, if it can go on.
    pub fn wear_new(&mut self, item: Item) -> Result<Fit, WearRefused> {
        let fit = self.check_wear_new(&item)?;
        self.worn.push(item);
        Ok(fit)
    }

    /// Whether this person may take this off, by what lies on it: anything not worn
    /// may go. What this says belongs to whoever is acting, and not to `move_item` or
    /// `hand_over`: a body falling, or an ability, isn't someone trying.
    pub fn may_remove(&self, id: Id, by: Remover) -> bool {
        let Some(item) = self.worn.iter().find(|item| item.id == id) else {
            return true;
        };
        match item.favour {
            None => true,
            Some(Favour::Blessed) => by == Remover::Wearer,
            Some(Favour::Cursed) => false,
        }
    }

    /// Takes out something to hand over, with all in it, unless it isn't carried.
    pub fn hand_over(&mut self, id: Id) -> Option<Item> {
        self.take(id)
    }

    /// Everything carried and worn, each with all in it, as what is left of a body.
    pub fn spilled(self) -> Vec<Item> {
        let carried = self.items.into_iter().map(|placed| placed.item);
        carried.chain(self.worn).collect()
    }

    /// How a worn item fits its wearer.
    pub fn worn_fit(&self, id: Id) -> Option<Fit> {
        let item = self.worn.iter().find(|item| item.id == id)?;
        let wear = item.kind.def().wear?;
        item.made
            .fit(wear.adjustable, self.creature.def().size, item.quality)
    }

    /// For a creature becoming another kind: what the new body still takes stays
    /// on, checked in the order it was put on so what is under goes back first.
    /// What it can't wear drops, cursed or not (owner, 2026-11-04), and what was
    /// carried goes in the new inventory, stacking, as far as it fits. Gives back
    /// what drops: what can't be worn and what there was no room for.
    pub fn change_body(&mut self, kind: CreatureKind, limits: StackLimits) -> Vec<Item> {
        let old = std::mem::take(self);
        self.creature = kind;
        self.outfitted = old.outfitted;
        self.size = kind.def().inventory;
        self.bin = old.bin;
        let mut dropped = Vec::new();
        for item in old.worn {
            if self.check_wear_new(&item).is_ok() {
                self.worn.push(item);
            } else {
                dropped.push(item);
            }
        }
        let carried = old.items.into_iter().map(|placed| placed.item);
        dropped.extend(carried.filter_map(|item| self.give(item, limits).err()));
        dropped
    }
}
