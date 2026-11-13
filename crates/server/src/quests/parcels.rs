//! A delivery's parcel (owner): the giver hands one over for each delivery
//! on taking the quest (refused if it doesn't fit), and arriving in the town
//! it is for delivers it. A quest item like any other: never given or traded,
//! destroyed if dropped, gone if the quest ends.

use shared::id::Id;
use shared::item::{Inventory, Item, ItemKind};
use shared::quest::{Quest, QuestId, Target};

use crate::ServerWorld;
use crate::ids::new_id;
use crate::sites::place_name_of;

/// The parcels a quest's giver hands over: one for each town it is taken to.
pub(super) fn parcels_for(world: &ServerWorld, quest: &Quest) -> Vec<Item> {
    (quest.steps.iter())
        .filter_map(|step| match step.target {
            Target::Town { chunk } => Some(chunk),
            _ => None,
        })
        .map(|chunk| {
            let town = world.pois.in_chunk(chunk);
            let name = town.and_then(|poi| place_name_of(world, &poi));
            Item {
                quest: Some(quest.id),
                name: name.map(|town| format!("Parcel for {town}")),
                made_at: Some(world.made_now()),
                ..Item::new(new_id(), ItemKind::Parcel)
            }
        })
        .collect()
}

/// Takes one of `quest`'s parcels from what is carried: delivered. Whether
/// there was one.
pub(super) fn deliver(inventory: &mut Inventory, quest: QuestId) -> bool {
    let Some(id) = parcel_in(inventory, quest) else {
        return false;
    };
    inventory.hand_over(id).is_some()
}

/// The id of one of `quest`'s parcels carried, at any depth.
fn parcel_in(inventory: &Inventory, quest: QuestId) -> Option<Id> {
    fn within(item: &Item, quest: QuestId) -> Option<Id> {
        if item.kind == ItemKind::Parcel && item.quest == Some(quest) {
            return Some(item.id);
        }
        (item.contents.iter()).find_map(|placed| within(&placed.item, quest))
    }
    let carried = inventory.items.iter().map(|placed| &placed.item);
    carried
        .chain(inventory.worn.iter())
        .find_map(|item| within(item, quest))
}
