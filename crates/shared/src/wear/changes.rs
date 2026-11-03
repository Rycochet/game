//! Putting things on and taking them off, handing them over, and changing body.

use crate::creature::CreatureKind;
use crate::id::Id;
use crate::item::{Favour, Inventory, Item};
use crate::wear::{Fit, Remover, WearRefused};

impl Inventory {
    /// Puts this carried item on, if it can go.
    pub fn wear(&mut self, id: Id) -> Result<Fit, WearRefused> {
        let fit = self.check_wear(id)?;
        let item = self.take(id).ok_or(WearRefused::NoSuchItem)?;
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

    /// Takes out something to hand over, with all in it, unless it isn't carried or
    /// is worn under something.
    pub fn hand_over(&mut self, id: Id) -> Option<Item> {
        if self.covered_by(id).is_some() {
            return None;
        }
        self.take(id)
    }

    /// Everything carried and worn, each with all in it, as what is left of a body.
    pub fn spilled(self) -> Vec<Item> {
        let pockets = self.pockets.into_iter().map(|placed| placed.item);
        pockets.chain(self.worn).collect()
    }

    /// How a worn item fits its wearer.
    pub fn worn_fit(&self, id: Id) -> Option<Fit> {
        let item = self.worn.iter().find(|item| item.id == id)?;
        let wear = item.kind.def().wear?;
        item.made
            .fit(wear.adjustable, self.creature.def().size, item.quality)
    }

    /// For a character becoming another kind of creature (dev tools): everything
    /// comes off and goes in the new pockets, as far as it fits. Gives back what
    /// didn't.
    pub fn change_body(&mut self, kind: CreatureKind) -> Vec<Item> {
        let old = std::mem::take(self);
        self.creature = kind;
        self.outfitted = old.outfitted;
        self.pockets_size = kind.def().pockets;
        let everything = old
            .pockets
            .into_iter()
            .map(|placed| placed.item)
            .chain(old.worn);
        everything
            .filter_map(|item| self.give(item).err())
            .collect()
    }
}
