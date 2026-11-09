//! What moving things costs in energy, on both sides.

use crate::energy::{covered_cost, dressing_cost, handling_cost};
use crate::id::Id;
use crate::item::{Holder, Inventory, Item};

impl Inventory {
    /// What putting on or taking off these together costs: each one's own, and
    /// getting at them past everything worn over any of them, once (owner,
    /// 2026-11-09: under something, things only take longer, never refuse).
    pub fn changing_cost(&self, items: &[&Item]) -> u32 {
        let own: u32 = items
            .iter()
            .map(|item| dressing_cost(item.total_weight()))
            .sum();
        let over = self.worn_over(items);
        if over.is_empty() {
            return own;
        }
        let grams = over.iter().map(|item| item.total_weight()).sum();
        own + covered_cost(grams)
    }

    /// What taking this off costs: nothing if it isn't worn.
    pub fn taking_off_cost(&self, id: Id) -> u32 {
        match self.worn.iter().find(|item| item.id == id) {
            Some(item) => self.changing_cost(&[item]),
            None => 0,
        }
    }

    /// What moving this carried item to this grid costs: nothing within a grid.
    pub fn move_cost(&self, id: Id, to: Holder) -> u32 {
        let Some(item) = self.find(id) else {
            return 0;
        };
        if self.is_worn(id) {
            self.taking_off_cost(id)
        } else if self.placed(id).is_some_and(|(from, _)| from == to) {
            0
        } else {
            handling_cost(item.total_weight())
        }
    }

    /// What putting this carried item down costs, or handing it over: taking it off
    /// first, if it is worn.
    pub fn put_down_cost(&self, id: Id) -> u32 {
        let Some(item) = self.find(id) else {
            return 0;
        };
        self.taking_off_cost(id) + handling_cost(item.total_weight())
    }
}
