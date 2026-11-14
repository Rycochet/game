//! Eating (`ClientMessage::Consume`): the top one of food carried, or lying in
//! reach, is taken and eaten (`Sheet::eat`), in turn with the player's steps.

use bevy::prelude::*;
use shared::bonus::is_food;
use shared::energy::handling_cost;
use shared::id::Id;
use shared::item::Item;
use shared::protocol::ServerMessage;

use super::Client;
use crate::ServerWorld;
use crate::ids::new_id;

/// Eats one of `item`, carried or on the ground. Whether the client is still
/// there, and what it cost (handling it), nothing if it was refused.
pub(super) fn consume(client: &mut Client, world: &mut ServerWorld, item: Id) -> (bool, u32) {
    let tick = world.action_clock.ticks();
    let carried = client.inventory.find(item).is_some();
    let taken = match carried {
        true => take_carried(client, item),
        false => crate::ground::take_food(client, world, item),
    };
    let cost = match taken {
        Ok(eaten) => {
            let (sheet, situation) = client.sheet_and_situation();
            sheet.eat(eaten.kind, situation, tick);
            client.unsaved = true;
            client.refresh_attributes(tick);
            handling_cost(eaten.total_weight())
        }
        Err(why) => {
            warn!("Refused eating {item:?}: {why}");
            0
        }
    };
    client.update_speed(tick);
    client.ground_told.again |= !carried;
    let told = client.send_inventory()
        && client
            .link
            .send(&ServerMessage::Sheet(client.sheet.as_sent()))
        && client.send_vitals(tick);
    (told, cost)
}

/// Takes the top one of a carried stack of food, to be eaten.
fn take_carried(client: &mut Client, item: Id) -> Result<Item, &'static str> {
    let found = client.inventory.find(item).ok_or("it isn't carried")?;
    if !is_food(found.top().kind) {
        return Err("it can't be eaten");
    }
    if found.holds_quest_item() || client.inventory.is_worn(item) {
        return Err("it can't be eaten from there");
    }
    let taken = client.inventory.take_one(item, &mut new_id);
    taken.ok_or("it can't be taken from there")
}
