//! Eating what is held by letting go of it on the clothes panel: the top one of
//! food (`bonus::is_food`), carried or from the ground, as the server will
//! (`ClientMessage::Consume`); the rest of its stack stays where it was.

use shared::bonus::is_food;
use shared::energy::handling_cost;
use shared::id::Id;
use shared::item::{Inventory, Item};
use shared::protocol::ClientMessage;

use super::super::Holding;
use super::Outbox;
use crate::common::{SpentEnergy, TookItem};
use crate::connection::ToServer;

/// The carried stack `held` is, or is part of (`Holding::group` or `counted`),
/// if it is food: its id, as its top one is what is eaten.
pub(super) fn carried_food(carried: &Inventory, holding: &Holding, held: Id) -> Option<Id> {
    let group = holding.group.as_ref().map(|(stack, _)| *stack);
    let counted = holding.counted.map(|(stack, _)| stack);
    let stack = group.or(counted).unwrap_or(held);
    let item = carried.find(stack)?;
    (is_food(item.top().kind) && !carried.is_worn(stack)).then_some(stack)
}

/// Eats the top one of this carried stack, at once as the server will.
pub(super) fn eat_carried(
    stack: Id,
    carried: &mut Inventory,
    stand_in: &mut impl FnMut() -> Id,
    out: &mut Outbox,
) {
    out.to_server
        .write(ToServer(ClientMessage::Consume { item: stack }));
    if let Some(eaten) = carried.take_one(stack, stand_in) {
        out.spent
            .write(SpentEnergy(handling_cost(eaten.total_weight())));
    }
}

/// Eats the top one of this, lying in a bag in reach (`with` is the ground):
/// the rest stays there, as the server will say.
pub(super) fn eat_from_ground(id: Id, mut item: Item, with: Id, out: &mut Outbox) {
    out.to_server
        .write(ToServer(ClientMessage::Consume { item: id }));
    // Its id doesn't matter: it is gone at once.
    let top = item.take_top(&mut || Id::stand_in(0));
    if top.is_none() {
        out.took.write(TookItem { with, item: id });
    }
    let eaten = top.unwrap_or(item);
    out.spent
        .write(SpentEnergy(handling_cost(eaten.total_weight())));
}
