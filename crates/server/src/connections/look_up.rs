//! Items asked after by id (`ClientMessage::LookUpItems`), for showing them
//! where they are written into text (`shared::item_text`). Anything loaded is
//! looked in: what players carry, and every open world's loaded sites. Each is
//! told as the player may know it, alone, and never where it is.

use bevy::prelude::*;
use shared::id::Id;
use shared::item::Item;
use shared::protocol::{MOST_LOOKED_UP, ServerMessage};

use super::{Client, Phase};
use crate::ServerWorlds;

impl Client {
    /// Notes these items to be looked up this tick, each once, and no more than
    /// `MOST_LOOKED_UP` in all.
    pub(super) fn ask_after(&mut self, ids: Vec<Id>) {
        for id in ids {
            if self.looking_up.contains(&id) {
                continue;
            }
            if self.looking_up.len() == MOST_LOOKED_UP {
                warn!("Ignored items asked after past the {MOST_LOOKED_UP} a tick");
                return;
            }
            self.looking_up.push(id);
        }
    }
}

/// Tells each client that asked what the items it asked after are.
pub(super) fn answer_look_ups(clients: &mut [Client], worlds: &mut ServerWorlds) {
    for i in 0..clients.len() {
        if clients[i].gone || clients[i].looking_up.is_empty() {
            continue;
        }
        let asked = std::mem::take(&mut clients[i].looking_up);
        let (mut found, mut unknown) = (Vec::new(), Vec::new());
        for id in asked {
            match find_loaded(clients, worlds, id) {
                Some(item) => found.push(item.alone()),
                None => unknown.push(id),
            }
        }
        let client = &mut clients[i];
        let found = client.as_seen(found);
        if !client
            .link
            .send(&ServerMessage::ItemsLookedUp { found, unknown })
        {
            let world = client.world;
            client.disconnect(worlds.of(world));
        }
    }
}

/// The item with this id wherever the server has it loaded.
fn find_loaded<'a>(clients: &'a [Client], worlds: &'a ServerWorlds, id: Id) -> Option<&'a Item> {
    let playing = clients
        .iter()
        .filter(|client| !client.gone && client.phase == Phase::Playing);
    let carried = playing
        .into_iter()
        .find_map(|client| client.inventory.find_within(id));
    carried.or_else(|| worlds.iter().find_map(|(_, world)| world.find_item(id)))
}
