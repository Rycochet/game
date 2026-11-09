//! A player putting something on, from what they carry or from the ground: what is
//! in its way comes off into their inventory (`shared::wear::Swapped`), and if it
//! can't go on they are told why (`ServerMessage::NotWorn`). It all takes the time
//! `Swapped::cost` says, longer under other things.

use shared::id::Id;
use shared::item::{Item, StackLimits};
use shared::protocol::ServerMessage;
use shared::wear::WearRefused;

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
                self.energy.spend(tick, self.speed, swapped.cost);
                true
            }
            Err(why) => {
                self.not_worn(top, why);
                false
            }
        }
    }

    /// Puts on something from the ground: what putting it on costs, with taking
    /// off what is in its way, or why it can't go on (the player has been told).
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
                Ok(swapped.cost)
            }
            Err(why) => {
                self.not_worn(id, why);
                Err(why)
            }
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
