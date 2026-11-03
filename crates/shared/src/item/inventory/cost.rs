//! What moving things costs in energy, on both sides.

use crate::energy::{dressing_cost, handling_cost};
use crate::id::Id;
use crate::item::{Holder, Inventory};

impl Inventory {
    /// What moving this carried item to this grid costs: nothing within a grid.
    pub fn move_cost(&self, id: Id, to: Holder) -> u32 {
        let Some(item) = self.find(id) else {
            return 0;
        };
        let grams = item.total_weight();
        if self.is_worn(id) {
            dressing_cost(grams)
        } else if self.placed(id).is_some_and(|(from, _)| from == to) {
            0
        } else {
            handling_cost(grams)
        }
    }

    /// What putting this carried item down costs, or handing it over: taking it off
    /// first, if it is worn.
    pub fn put_down_cost(&self, id: Id) -> u32 {
        let Some(item) = self.find(id) else {
            return 0;
        };
        let grams = item.total_weight();
        let taking_off = if self.is_worn(id) {
            dressing_cost(grams)
        } else {
            0
        };
        taking_off + handling_cost(grams)
    }
}
