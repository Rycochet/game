//! A new character's first sheet and clothes, and welcoming a character into the world.

use shared::creature::CreatureKind;
use shared::item::{Inventory, Item, ItemKind, MadeAt, Quality, StackLimits};
use shared::item_form::{ItemOrder, make_item};
use shared::protocol::ServerMessage;
use shared::sheet::{Appearance, Build, Sheet};

use super::{Client, Phase};
use crate::sites::{self};
use crate::{ServerOptions, ServerWorld, world_map};

/// A new item of this kind, made for this size or the wearer's, and their fingers.
/// Kinds that don't come in sizes are `Medium`, whatever is asked for.
pub(crate) fn made_for(
    inventory: &Inventory,
    kind: ItemKind,
    size: Option<shared::creature::Size>,
    magical: bool,
    quality: Quality,
    made_at: MadeAt,
) -> Item {
    let order = ItemOrder {
        size,
        magical,
        ..ItemOrder::plain(kind, quality)
    };
    let id = crate::ids::new_id();
    let made_at = Some(made_at);
    make_item(
        &order,
        inventory.creature,
        id,
        id.seed(),
        made_at,
        &mut crate::ids::new_id,
    )
}

/// What a joining character is like: as saved (with attributes worked out for a
/// save from before there were nine); or as they asked when being made; or if they
/// didn't ask, plain for their kind.
pub(super) fn start_sheet(
    saved: Option<Sheet>,
    asked: Option<(CreatureKind, Appearance, Build)>,
    inventory: &mut Inventory,
) -> Sheet {
    if let Some(sheet) = saved {
        return sheet.settled(inventory.creature);
    }
    match asked {
        Some((kind, appearance, build)) => {
            inventory.change_body(kind, StackLimits::NONE);
            Sheet {
                appearance,
                ..Sheet::built(kind, build)
            }
        }
        None => Sheet::new(inventory.creature),
    }
}

/// A new character's clothes.
pub(super) fn give_first_clothes(inventory: &mut Inventory, made_at: MadeAt) {
    if std::mem::replace(&mut inventory.outfitted, true) {
        return;
    }
    for kind in [ItemKind::TShirt, ItemKind::Trousers, ItemKind::Shoes] {
        // Straight on, as the inventory may be full. Only for bodies they fit: other
        // kinds of creature start with none.
        if inventory.creature == shared::creature::CreatureKind::Human {
            let item = made_for(inventory, kind, None, false, Quality::Common, made_at);
            inventory.worn.push(item);
        }
    }
}

/// Loads the character (or starts a new one), and tells the client the
/// world and where they are; back into the site level they were on, if any.
pub(super) fn welcome(
    client: &mut Client,
    world: &mut ServerWorld,
    options: &ServerOptions,
) -> bool {
    let id = client
        .character_id
        .as_ref()
        .expect("decided before joining");
    let loaded = world
        .storage
        .load_character(id, world.seed.id())
        .and_then(|character| {
            let inventory = world.storage.load_inventory(id)?;
            let gates = world.storage.load_gates(id)?;
            Ok((character, inventory, world.storage.load_sheet(id)?, gates))
        });
    // Not `Playing` yet, so nothing blank can be saved over the real character.
    match loaded {
        Ok((character, inventory, sheet, gates)) => {
            (client.character, client.other_worlds) = character;
            client.gates = gates;
            // Never saved in this world (or forgotten): new here, so they
            // arrive through one of its World Gates.
            if client.character.forgotten_to == 0 {
                client.character.world_position = crate::world_map::arrival(world, id);
            }
            client.inventory = inventory;
            client.sheet = start_sheet(sheet, client.starting_as.take(), &mut client.inventory);
            give_first_clothes(&mut client.inventory, world.made_now());
            client.speed = client.sheet.speed(&client.inventory);
            client.unsaved = true;
        }
        Err(error) => return client.refuse_for_saves(&error),
    }
    client.phase = Phase::Playing;
    if !send_welcome(client, world, options) {
        return false;
    }
    match (client.character.visit.take(), client.character.hall) {
        (Some(visit), _) => sites::resume(client, world, visit),
        (None, Some(at)) => crate::hall::enter(client, world, at),
        (None, None) => world_map::look_around(client, world),
    }
}

/// Tells the client the world they are in and where on its world map, then
/// what they are like, the gates they have gone through and what they carry.
/// `false` if the client has gone.
pub(super) fn send_welcome(
    client: &mut Client,
    world: &ServerWorld,
    options: &ServerOptions,
) -> bool {
    let (x, y) = client.world_position();
    let welcome = ServerMessage::Welcome {
        world: world.info(),
        time: world.time(),
        server_clock: world.server_clock.seconds(),
        x,
        y,
        dev_tools: options.dev_tools_for(client.host),
        can_pause: options.pausable && client.host,
        stacks: world.stacks,
    };
    client.link.send(&welcome)
        && client
            .link
            .send(&ServerMessage::Sheet(client.sheet.clone()))
        && client.send_visited_gates()
        && client.send_inventory()
}
