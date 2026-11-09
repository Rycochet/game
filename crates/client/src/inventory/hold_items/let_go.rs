//! Letting go of what is held: taking it from someone, wearing, dropping, giving,
//! binning or moving it, as aimed. The server is told and decides; what it is
//! sure to agree with is shown at once.

use bevy::prelude::*;
use shared::energy::handling_cost;
use shared::item::Item;
use shared::protocol::ClientMessage;

use super::super::hold::{Target, stack_onto};
use super::super::wear_note::try_wearing;
use super::super::{Carried, HeldBy, Holding};
use super::Outbox;
use crate::common::{Dressing, GROUND, GaveItem, HeldOverGifts, HeldStock, SpentEnergy, TookItem};
use crate::connection::ToServer;

/// Lets go of it, as aimed.
pub(super) fn let_go_of(
    held: HeldBy,
    aim: Target,
    holding: &mut ResMut<Holding>,
    carried: &mut ResMut<Carried>,
    (over_gifts, held_stock): (&HeldOverGifts, &mut ResMut<HeldStock>),
    out: &mut Outbox,
) {
    let Outbox {
        to_server,
        gave,
        took,
        spent,
        dressing,
        stacks,
        wear: (note, god, time),
        ..
    } = out;
    if aim == (Target::Body { allowed: false }) {
        let new = holding.stock.as_ref().map(|(_, item)| item);
        let tried = try_wearing(&carried.0, (held.item.id, new), god.0, stacks.0);
        if let Err(why) = tried {
            note.say(why, &carried.0, time.elapsed_secs_f64());
        }
    }
    if let Some((with, item)) = holding.stock.take() {
        held_stock.set_if_neq(HeldStock(None));
        if let Some(gift) = over_gifts.0 {
            // Moved about in their grid: nothing to ask anyone.
            gave.write(GaveItem(gift));
        } else if aim == Target::Bin {
            // Only from the ground: see `aim`.
            let id = held.item.id;
            carried.0.to_bin_new(item);
            to_server.write(ToServer(ClientMessage::BinFromGround { item: id }));
            took.write(TookItem { with, item: id });
        } else if aim == (Target::Body { allowed: true }) {
            let id = held.item.id;
            to_server.write(ToServer(ClientMessage::WearFromGround { item: id }));
            // Only the top of a stack goes on, and the rest stays on the ground.
            let mut item = item;
            let top = item.take_top();
            let all_of_it = top.is_none();
            let wearing = top.unwrap_or(item);
            let grams = wearing.total_weight();
            // Otherwise only the server can say. What is in its way comes off.
            if let Ok(swapped) = carried.0.wear_new_swapping(wearing, god.0, stacks.0) {
                if all_of_it {
                    took.write(TookItem { with, item: id });
                }
                spent.write(SpentEnergy(swapped.cost + handling_cost(grams)));
                dressing.write(Dressing(swapped.cost));
            }
        } else if let Some(onto) = stack_onto(aim).filter(|_| with == GROUND) {
            let id = held.item.id;
            let grams = item.total_weight();
            to_server.write(ToServer(ClientMessage::StackFromGround { item: id, onto }));
            // What is left stays on the ground, as the server will say.
            if let Ok(rest) = carried.0.add_to_stack(item, onto, stacks.0) {
                let left = rest.as_ref().map_or(0, Item::total_weight);
                if rest.is_none() {
                    took.write(TookItem { with, item: id });
                }
                spent.write(SpentEnergy(handling_cost(grams - left)));
            }
        } else if let Some((to, x, y, rotation)) =
            super::super::hold::drop_move(&carried.0, held.item, aim)
        {
            let grams = item.total_weight();
            if carried.0.add(item, to, x, y, rotation).is_ok() {
                let item = held.item.id;
                to_server.write(ToServer(if with == GROUND {
                    ClientMessage::TakeFromGround {
                        item,
                        to,
                        x,
                        y,
                        rotation,
                    }
                } else {
                    ClientMessage::TakeItem {
                        with,
                        item,
                        to,
                        x,
                        y,
                        rotation,
                    }
                }));
                took.write(TookItem { with, item });
                spent.write(SpentEnergy(handling_cost(grams)));
            }
        }
        return;
    }
    // Some of a mixed stack: whatever is asked is for those items together.
    let group = holding.group.take();
    let mut send = |ToServer(then): ToServer| {
        to_server.write(ToServer(match &group {
            Some((stack, items)) => ClientMessage::FromStack {
                stack: *stack,
                items: items.clone(),
                then: Box::new(then),
            },
            None => then,
        }));
    };
    if aim == (Target::Body { allowed: true }) {
        let item = held.item.id;
        // Only the top of a stack goes on.
        send(ToServer(ClientMessage::WearItem { item }));
        // Otherwise only the server can say. What is in its way comes off.
        if let Ok(swapped) = carried.0.wear_swapping(item, god.0, stacks.0) {
            spent.write(SpentEnergy(swapped.cost));
            dressing.write(Dressing(swapped.cost));
        }
        return;
    }
    if aim == (Target::Drop { allowed: true }) {
        let item = held.item.id;
        let cost = carried.0.put_down_cost(item);
        let taking_off = carried.0.taking_off_cost(item);
        if carried.0.hand_over(item).is_some() {
            send(ToServer(ClientMessage::DropItem { item }));
            spent.write(SpentEnergy(cost));
            dressing.write(Dressing(taking_off));
        }
        return;
    }
    if let Target::Someone(to) = aim {
        let item = held.item.id;
        let cost = carried.0.put_down_cost(item);
        let taking_off = carried.0.taking_off_cost(item);
        if carried.0.hand_over(item).is_some() {
            dressing.write(Dressing(taking_off));
            send(ToServer(if to == GROUND {
                ClientMessage::DropItem { item }
            } else {
                ClientMessage::GiveItem { item, to }
            }));
            spent.write(SpentEnergy(cost));
            if let Some(gift) = over_gifts.0 {
                gave.write(GaveItem(gift));
            }
        }
        return;
    }
    if aim == Target::Bin {
        let item = held.item.id;
        let taking_off = carried.0.taking_off_cost(item);
        if carried.0.to_bin(item) {
            send(ToServer(ClientMessage::BinItem { item }));
            spent.write(SpentEnergy(taking_off));
            dressing.write(Dressing(taking_off));
        }
        return;
    }
    if let Some(onto) = stack_onto(aim) {
        let item = held.item.id;
        let to = carried.0.placed(onto).map(|(holder, _)| holder);
        let cost = to.map_or(0, |to| carried.0.move_cost(item, to));
        let taking_off = carried.0.taking_off_cost(item);
        if carried.0.stack_onto(item, onto, stacks.0).is_ok() {
            spent.write(SpentEnergy(cost));
            dressing.write(Dressing(taking_off));
            send(ToServer(ClientMessage::StackItem { item, onto }));
        }
        return;
    }
    if let Some((to, x, y, rotation)) = super::super::hold::drop_move(&carried.0, held.item, aim) {
        // Shown at once; `Inventory` follows, putting it right if the server disagrees.
        let item = held.item.id;
        let cost = carried.0.move_cost(item, to);
        let taking_off = carried.0.taking_off_cost(item);
        if carried.0.move_item(item, to, x, y, rotation).is_ok() {
            spent.write(SpentEnergy(cost));
            dressing.write(Dressing(taking_off));
            send(ToServer(ClientMessage::MoveItem {
                item,
                to,
                x,
                y,
                rotation,
            }));
        }
    }
}
