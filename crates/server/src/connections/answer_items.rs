//! Answering item moves: dropping, taking, wearing, giving, binning and moving things.

use bevy::prelude::*;
use shared::energy::handling_cost;
use shared::item::{Holder, Item};
use shared::protocol::{ClientMessage, ServerMessage};

use super::{Action, Client};
use crate::ServerWorld;
use crate::ground::GroundAction;
use crate::sites::{self};

/// Answers a message about items. Whether the client is still welcome.
pub(super) fn answer_items(
    client: &mut Client,
    message: ClientMessage,
    world: &mut ServerWorld,
) -> bool {
    // Some of a mixed stack: lifted out together, then dealt with as anything is.
    let message = match message {
        ClientMessage::FromStack { stack, items, then } => match *then {
            ClientMessage::DropItem { item } => {
                let group = Some((stack, items));
                let action = Action::Ground(GroundAction::Drop { item, group });
                client.waiting.push_back(action);
                return true;
            }
            then @ (ClientMessage::MoveItem { .. }
            | ClientMessage::StackItem { .. }
            | ClientMessage::GiveItem { .. }
            | ClientMessage::WearItem { .. }
            | ClientMessage::BinItem { .. }) => {
                client.inventory.lift_group(stack, &items);
                then
            }
            other => {
                warn!("Refused {other:?} for part of a stack");
                return client.send_inventory();
            }
        },
        other => other,
    };
    // An id from inside a stack stands for it and all above it: lifted out on its
    // own, it moves like anything else, and goes back if it isn't moved.
    if let ClientMessage::MoveItem { item, .. }
    | ClientMessage::StackItem { item, .. }
    | ClientMessage::GiveItem { item, .. }
    | ClientMessage::WearItem { item }
    | ClientMessage::BinItem { item } = message
    {
        client.inventory.lift_part(item);
    }
    match message {
        ClientMessage::DropItem { item } => {
            let action = Action::Ground(GroundAction::Drop { item, group: None });
            client.waiting.push_back(action);
            true
        }
        ClientMessage::TakeFromGround {
            item,
            to,
            x,
            y,
            rotation,
        } => {
            client.waiting.push_back(Action::Ground(GroundAction::Take {
                item,
                to,
                x,
                y,
                rotation,
            }));
            true
        }
        ClientMessage::StackFromGround { item, onto } => {
            let action = Action::Ground(GroundAction::Stack { item, onto });
            client.waiting.push_back(action);
            true
        }
        ClientMessage::WearFromGround { item } => {
            let action = Action::Ground(GroundAction::Wear { item });
            client.waiting.push_back(action);
            true
        }
        ClientMessage::BinFromGround { item } => {
            let action = Action::Ground(GroundAction::Bin { item });
            client.waiting.push_back(action);
            true
        }
        ClientMessage::MoveItem {
            item,
            to,
            x,
            y,
            rotation,
        } => {
            let cost = client.inventory.move_cost(item, to);
            let tick = world.action_clock.ticks();
            if client.may_take_off(item) {
                match client.inventory.move_item(item, to, x, y, rotation) {
                    Ok(()) => {
                        client.unsaved = true;
                        client.energy.spend(tick, client.speed, cost);
                    }
                    Err(why) => warn!("Refused moving item {item:?} to {to:?}: {why:?}"),
                }
            }
            client.update_speed(world.action_clock.ticks());
            client.send_inventory()
        }
        ClientMessage::StackItem { item, onto } => {
            let to = client.inventory.placed(onto);
            let to = to.map_or(Holder::Inventory, |(holder, _)| holder);
            let cost = client.inventory.move_cost(item, to);
            let tick = world.action_clock.ticks();
            if client.may_take_off(item) {
                match client.inventory.stack_onto(item, onto, world.stacks) {
                    Ok(_) => {
                        client.unsaved = true;
                        client.energy.spend(tick, client.speed, cost);
                    }
                    Err(why) => warn!("Refused stacking {item:?} on {onto:?}: {why:?}"),
                }
            }
            client.update_speed(world.action_clock.ticks());
            client.send_inventory()
        }
        ClientMessage::SortGrid { holder } => {
            // Rearranging one grid takes no time.
            if client.inventory.sort_grid(holder, world.stacks) {
                client.unsaved = true;
            } else {
                debug!("Nothing sorted in {holder:?}");
            }
            client.send_inventory()
        }
        ClientMessage::GiveItem { item, to } => {
            let mut stock = None;
            let cost = client.inventory.put_down_cost(item);
            let given = if !client.may_take_off(item) {
                Err("it won't come off")
            } else if sites::keeper_building(client, world, to).is_some() {
                sites::give_to_keeper(client, world, item, to).map(|items| stock = Some(items))
            } else {
                crate::entities::give(client, world, item, to)
            };
            match given {
                Ok(()) => {
                    client.unsaved = true;
                    let tick = world.action_clock.ticks();
                    client.energy.spend(tick, client.speed, cost);
                }
                Err(why) => warn!("Refused giving {item:?} to {to:?}: {why}"),
            }
            client.update_speed(world.action_clock.ticks());
            let sent = client.send_inventory();
            match stock {
                Some(items) => {
                    let items = client.as_seen(items);
                    sent && client
                        .link
                        .send(&ServerMessage::TradeStock { with: to, items })
                }
                None => sent,
            }
        }
        ClientMessage::TakeItem {
            with,
            item,
            to,
            x,
            y,
            rotation,
        } => {
            match sites::take_from_keeper(client, world, with, item, (to, x, y, rotation)) {
                Ok(()) => {
                    client.unsaved = true;
                    let grams = client.inventory.find(item).map_or(0, Item::total_weight);
                    let tick = world.action_clock.ticks();
                    client
                        .energy
                        .spend(tick, client.speed, handling_cost(grams));
                }
                Err(why) => warn!("Refused taking {item:?} from {with:?}: {why}"),
            }
            client.update_speed(world.action_clock.ticks());
            let stock = sites::keeper_stock(client, world, with);
            let sent = client.send_inventory();
            match stock {
                Some(items) => {
                    let items = client.as_seen(items);
                    sent && client.link.send(&ServerMessage::TradeStock { with, items })
                }
                None => sent,
            }
        }
        ClientMessage::WearItem { item } => {
            let tick = world.action_clock.ticks();
            // What is in its way comes off, or the player is told why not.
            client.put_on(item, world.stacks, tick);
            client.update_speed(world.action_clock.ticks());
            client.send_inventory()
        }
        ClientMessage::BinItem { item } => {
            if !client.may_take_off(item) {
                // Nothing more to say: it stays on.
            } else if client.inventory.to_bin(item) {
                client.unsaved = true;
            } else {
                warn!("Refused throwing away {item:?}");
            }
            client.update_speed(world.action_clock.ticks());
            client.send_inventory()
        }
        _ => unreachable!("`answer` only passes item messages here"),
    }
}
