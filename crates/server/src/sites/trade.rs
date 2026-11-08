//! Opening trade with a shopkeeper.

use bevy::prelude::*;
use shared::id::Id;
use shared::item::{Item, MadeAt, Quality};
use shared::item_form::give_form;
use shared::protocol::ServerMessage;
use shared::rng::Seed;
use shared::shop::{Shop, catch_up, sells};

use super::shop_here::shop_of;
use crate::ServerWorld;
use crate::connections::Client;
use crate::ids::new_id;

/// Opens trading with a shop's keeper for a player beside its door: its stock is
/// caught up and sent. Anything else is refused, with no answer. `false` if the
/// client has gone.
pub(crate) fn open_trade(client: &mut Client, world: &mut ServerWorld, with: Id) -> bool {
    let Some(visit) = &client.visit else {
        warn!("Refused opening a trade from outside any site");
        return true;
    };
    let position = visit.position;
    let Some(here) = shop_of(client, world, with) else {
        warn!("Refused opening a trade with someone who isn't here: {with:?}");
        return true;
    };
    let town = &here.building;
    if sells(town.look.kind).is_empty() || !here.in_reach(position) {
        warn!("Refused opening {:?} from {position:?}", town.look.kind);
        return true;
    }
    let (kind, quality, trades) = (town.look.kind, town.quality, town.trades);
    let (building, of_kind) = (town.look.id, here.of_kind);
    let seed = here.poi.seed.derive("shop");
    let seed = Seed::new(seed.at(&[here.index as i64, building as i64]));
    let (id, index) = (here.id, here.index);
    let now = world.minutes();
    let made = world.made_now();
    let level = world.site_mut(id).levels.entry(index).or_default();
    let keeping = level.records.entry(with).or_default();
    let saved = level.buildings.entry(building).or_default();
    let shop = Shop {
        kind,
        quality,
        trades: trades + saved.trades,
        of_kind,
    };
    let stock: Vec<Quality> = keeping
        .carried
        .iter()
        .map(|item| item.quality.unwrap_or_default())
        .collect();
    // The clock can be moved back (dev tools).
    let last_visit = saved.last_interaction.map(|at| at.max(0.0) as u64);
    let restocked = catch_up(
        seed,
        &shop,
        &stock,
        saved.last_made,
        last_visit,
        made.minute,
    );
    let mut items: Vec<Item> = restocked
        .keep
        .iter()
        .map(|&index| keeping.carried[index].clone())
        .collect();
    items.extend(restocked.new.iter().map(|new| {
        let mut item = Item {
            quality: new.quality,
            made_at: Some(MadeAt {
                minute: new.minute,
                ..made
            }),
            ..Item::new(new_id(), new.kind)
        };
        give_form(&mut item, None, &mut new_id);
        item.roll_secrets();
        item
    }));
    keeping.carried.clone_from(&items);
    saved.last_made = restocked.last_made;
    saved.last_interaction = Some(now);
    let items = client.as_seen(items);
    client.link.send(&ServerMessage::TradeStock { with, items })
}
