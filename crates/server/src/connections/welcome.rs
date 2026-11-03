//! A new character's first sheet and clothes, and welcoming a character into the world.

use shared::creature::CreatureKind;
use shared::item::{Inventory, Item, ItemKind, Made, MadeAt, Quality};
use shared::item_form::give_form;
use shared::protocol::ServerMessage;
use shared::sheet::{Appearance, Sheet};

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
    let body = inventory.creature.def();
    let size = size.filter(|_| kind.comes_in_sizes());
    // What is worn is made to fit the wearer unless told otherwise.
    let size = size.or(kind.def().wear.map(|_| body.size));
    let id = crate::ids::new_id();
    let quality = kind.def().has_quality.then_some(quality);
    let name = (quality == Some(Quality::Artifact))
        .then(|| shared::item::artifact_name(&mut id.seed().rng(), kind));
    let mut item = Item {
        made: Made::for_body(kind, size.unwrap_or_default(), body.fingers(), magical),
        quality,
        made_at: Some(made_at),
        name,
        ..Item::new(id, kind)
    };
    give_form(&mut item, size, &mut crate::ids::new_id);
    item
}

/// What a joining character is like: as saved; or as they asked when being made;
/// or if they didn't ask, plain for their kind.
pub(super) fn start_sheet(
    saved: Option<Sheet>,
    asked: Option<(CreatureKind, Appearance)>,
    inventory: &mut Inventory,
) -> Sheet {
    if let Some(sheet) = saved {
        return sheet;
    }
    match asked {
        Some((kind, appearance)) => {
            inventory.change_body(kind);
            Sheet {
                appearance,
                ..Sheet::new(kind)
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
        // Straight on, as the pockets may be full. Only for bodies they fit: other
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
            Ok((character, inventory, world.storage.load_sheet(id)?))
        });
    // Not `Playing` yet, so nothing blank can be saved over the real character.
    match loaded {
        Ok((character, inventory, sheet)) => {
            (client.character, client.other_worlds) = character;
            client.inventory = inventory;
            client.sheet = start_sheet(sheet, client.starting_as.take(), &mut client.inventory);
            give_first_clothes(&mut client.inventory, world.made_now());
            client.speed = client.sheet.speed(&client.inventory);
            client.unsaved = true;
        }
        Err(error) => return client.refuse_for_saves(&error),
    }
    client.phase = Phase::Playing;
    let (x, y) = client.world_position();
    let welcome = ServerMessage::Welcome {
        world: world.info(),
        clock: world.clock.minutes(),
        snow_line: world.snow_line,
        x,
        y,
        dev_tools: options.dev_tools_for(client.host),
        can_pause: options.pausable && client.host,
    };
    if !client.link.send(&welcome)
        || !client
            .link
            .send(&ServerMessage::Sheet(client.sheet.clone()))
        || !client.send_inventory()
    {
        return false;
    }
    match client.character.visit.take() {
        Some(visit) => sites::resume(client, world, visit),
        None => world_map::look_around(client, world),
    }
}
