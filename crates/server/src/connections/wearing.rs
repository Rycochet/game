//! A player putting something on, from what they carry or from the ground: what is
//! in its way comes off into their inventory (`shared::wear::Swapped`), and if it
//! can't go on they are told why (`ServerMessage::NotWorn`).

use shared::energy::dressing_cost;
use shared::id::Id;
use shared::item::{Item, StackLimits};
use shared::protocol::ServerMessage;
use shared::wear::{Swapped, WearRefused};

use super::Client;

impl Client {
    /// Puts on this carried item (the top of its stack), at this tick. Whether it
    /// went on.
    pub(super) fn put_on(&mut self, item: Id, limits: StackLimits, tick: u64) -> bool {
        let top = self
            .inventory
            .find(item)
            .map_or(item, |found| found.top().id);
        let tried = self.inventory.wear_swapping(item, self.god_mode, limits);
        match tried {
            Ok(swapped) => {
                self.unsaved = true;
                self.inventory.magic_found_out(top);
                self.spend_dressing(top, tick);
                self.spend_taking_off(&swapped, tick);
                true
            }
            Err(why) => {
                self.not_worn(top, why);
                false
            }
        }
    }

    /// Puts on something from the ground: what taking off for it costs, or why
    /// it can't go on (the player has been told).
    pub(crate) fn put_on_new(
        &mut self,
        item: Item,
        limits: StackLimits,
    ) -> Result<u32, WearRefused> {
        let id = item.id;
        let tried = self
            .inventory
            .wear_new_swapping(item, self.god_mode, limits);
        match tried {
            Ok(swapped) => {
                self.inventory.magic_found_out(id);
                Ok(self.taking_off_cost(&swapped))
            }
            Err(why) => {
                self.not_worn(id, why);
                Err(why)
            }
        }
    }

    fn taking_off_cost(&self, swapped: &Swapped) -> u32 {
        let grams = |id| self.inventory.find(id).map_or(0, Item::total_weight);
        swapped
            .taken_off
            .iter()
            .map(|&id| dressing_cost(grams(id)))
            .sum()
    }

    fn spend_taking_off(&mut self, swapped: &Swapped, tick: u64) {
        let cost = self.taking_off_cost(swapped);
        if cost > 0 {
            self.energy.spend(tick, self.speed, cost);
        }
    }

    /// Tells the player why it didn't go on. A curse gives itself away by holding.
    fn not_worn(&mut self, item: Id, why: WearRefused) {
        if let WearRefused::Stuck(stuck) = why {
            self.inventory.favour_found_out(stuck);
            self.unsaved = true;
        }
        self.link.send(&ServerMessage::NotWorn { item, why });
    }
}
