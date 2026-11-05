//! Stacks: items that look the same carried as one, in one place. The first item
//! in is the bottom, whose id is the stack's, and holds the rest in `Item::above`;
//! the last in is the top, which is what shows and the first to come off.

use serde::{Deserialize, Serialize};

use crate::id::Id;
use crate::item::{Item, Known};
use crate::item_form::ItemForm;

/// How big a stack may get. The server's to set (`HOW_STACK_GRAMS_PER_CELL`,
/// `HOW_STACK_MAX`), and it tells clients in `Welcome`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct StackLimits {
    /// The most a stack may weigh for each cell of its shape, in grams.
    pub grams_per_cell: u32,
    /// The most items in one stack, whatever they weigh.
    pub most: u32,
}

impl Default for StackLimits {
    fn default() -> Self {
        Self {
            grams_per_cell: 2000,
            most: 99,
        }
    }
}

impl StackLimits {
    /// Limits nothing can stack under.
    pub const NONE: Self = Self {
        grams_per_cell: 0,
        most: 1,
    };

    /// The most of this item one stack could hold: 1 for what can't stack.
    pub fn most_of(&self, item: &Item) -> u32 {
        if item.kind.def().container.is_some() {
            return 1;
        }
        let cells = item.shape().cells().count() as u32;
        let most_grams = self.grams_per_cell.saturating_mul(cells);
        let one = item.own_weight().max(1);
        (most_grams / one).clamp(1, self.most.max(1))
    }

    /// How many of `adding`'s items, taken from its top, could go on `stack`: 0 if
    /// they don't all look the same as it, or it is full.
    pub fn room(&self, stack: &Item, adding: &Item) -> u32 {
        if stack.id == adding.id
            || !stack.is_even()
            || !adding.members().all(|one| one.looks_like(stack))
        {
            return 0;
        }
        let cells = stack.shape().cells().count() as u32;
        let most_grams = self.grams_per_cell.saturating_mul(cells);
        let mut grams = stack.total_weight();
        let mut count = stack.count();
        let mut fits = 0;
        for one in adding.members().rev() {
            grams += one.own_weight();
            count += 1;
            if count > self.most || grams > most_grams {
                break;
            }
            fits += 1;
        }
        fits
    }
}

impl Item {
    /// How many items it is: 1 unless it is a stack.
    pub fn count(&self) -> u32 {
        1 + self.above.len() as u32
    }

    /// What lies on top, which is what a stack shows: itself if alone.
    pub fn top(&self) -> &Item {
        self.above.last().unwrap_or(self)
    }

    /// Every item in it, from the bottom up.
    pub fn members(&self) -> impl DoubleEndedIterator<Item = &Item> {
        std::iter::once(self).chain(&self.above)
    }

    /// Its items in groups that look the same, each from the top down, the group of
    /// its top first: one group unless it is mixed.
    pub fn groups(&self) -> Vec<Vec<&Item>> {
        let mut groups: Vec<Vec<&Item>> = Vec::new();
        for one in self.members().rev() {
            match groups.iter_mut().find(|group| group[0].looks_like(one)) {
                Some(group) => group.push(one),
                None => groups.push(vec![one]),
            }
        }
        groups
    }

    /// Whether every item in it looks the same as the others.
    pub fn is_even(&self) -> bool {
        self.above.iter().all(|one| one.looks_like(self))
    }

    /// Whether these two could share a stack, by what is known of them: neither
    /// can hold anything, and they differ in nothing anyone could see but their
    /// ids, when they were made and what is stacked on them. Going by what is
    /// known, the server's answer gives nothing away.
    pub fn looks_like(&self, other: &Item) -> bool {
        self.kind.def().container.is_none() && self.as_compared() == other.as_compared()
    }

    /// Takes the top off a stack. `None` if it is alone.
    pub fn take_top(&mut self) -> Option<Item> {
        self.above.pop()
    }

    /// Takes the top `count` off a stack, as a stack of their own. Only for fewer
    /// than it holds.
    pub fn split_top(&mut self, count: u32) -> Item {
        assert!(
            count > 0 && count < self.count(),
            "a split leaves both parts"
        );
        let mut taken = self.above.split_off(self.above.len() - count as usize);
        let mut bottom = taken.remove(0);
        bottom.above = taken;
        bottom
    }

    /// Puts all of `part` on top, keeping its order.
    pub fn put_on(&mut self, mut part: Item) {
        let above = std::mem::take(&mut part.above);
        self.above.push(part);
        self.above.extend(above);
    }

    /// What one item of it weighs, with anything in it, but not what is stacked on it.
    pub fn own_weight(&self) -> u32 {
        self.total_weight() - self.above.iter().map(Item::total_weight).sum::<u32>()
    }

    /// Whether `id` is one of the items stacked on this one.
    pub(super) fn has_above(&self, id: Id) -> bool {
        self.above.iter().any(|one| one.id == id)
    }

    /// What is compared to decide whether two look the same: as a client would see
    /// it, without what differs between any two items.
    fn as_compared(&self) -> Item {
        let mut copy = self.clone();
        copy.hide_secrets();
        // What is known shows already, and knowing there is nothing shows nothing.
        copy.known = Known::default();
        copy.id = Id::NOBODY;
        copy.made_at = None;
        copy.above.clear();
        if let Some(ItemForm::Ring(ring)) = &mut copy.form {
            for gem in ring
                .settings
                .iter_mut()
                .filter_map(|setting| setting.gem.as_mut())
            {
                *gem = gem.as_compared();
            }
        }
        copy
    }
}
