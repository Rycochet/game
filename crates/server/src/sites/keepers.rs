//! Shopkeepers: who keeps which building, and giving and taking their stock.

use bevy::prelude::*;
use shared::id::Id;
use shared::item::Item;
use shared::shop::sells;

use super::Mirror;
use super::shop_here::shop_of;
use crate::ServerWorld;
use crate::connections::Client;
use crate::ids::new_id;
use crate::storage::SiteId;

/// Who keeps each of a town level's shops, by building. Each has one for good, made
/// the first time it is needed.
pub(super) fn keepers_of(world: &mut ServerWorld, visit: &NewKeepers) -> Vec<(u16, Id)> {
    let saved = world.site_mut(visit.id).levels.entry(visit.index);
    let saved = saved.or_default();
    let shops = visit.shops.iter().map(|&building| {
        let save = saved.buildings.entry(building).or_default();
        let keeper = *save.keeper.get_or_insert_with(new_id);
        let old_stock = std::mem::take(&mut save.stock);
        let record = saved.records.entry(keeper).or_default();
        record.carried.extend(old_stock);
        (building, keeper)
    });
    shops.collect()
}

/// A town level, for `keepers_of`.
pub(super) struct NewKeepers {
    pub(super) id: SiteId,
    pub(super) index: u32,
    pub(super) shops: Vec<u16>,
}

/// Who keeps the shops of the towns set into a wilderness window (`mirrors`),
/// by building, made as `keepers_of` does: they are the towns' own.
pub(super) fn keepers_in_window(world: &mut ServerWorld, mirrors: &[Mirror]) -> Vec<(u16, Id)> {
    let mut keepers = Vec::new();
    for mirror in mirrors {
        let SiteId::Poi(chunk) = mirror.id else {
            continue;
        };
        let Some(wild) = world.wilderness.chunk(chunk) else {
            continue;
        };
        let shops = wild.town.iter().filter(|b| !sells(b.look.kind).is_empty());
        let shops: Vec<u16> = shops.map(|b| b.look.id).collect();
        if shops.is_empty() {
            continue;
        }
        let town = NewKeepers {
            id: mirror.id,
            index: mirror.index,
            shops,
        };
        keepers.extend(keepers_of(world, &town));
    }
    keepers
}

/// Which building on the player's level this is the keeper of, if any.
pub(crate) fn keeper_building(client: &Client, world: &mut ServerWorld, with: Id) -> Option<u16> {
    shop_of(client, world, with).map(|shop| shop.building.look.id)
}

/// Gives a shop's keeper something, from beside its door: it joins what the shop has,
/// which comes back, and the keeper remembers it. Selling, until there is payment.
pub(crate) fn give_to_keeper(
    client: &mut Client,
    world: &mut ServerWorld,
    item: Id,
    with: Id,
) -> Result<Vec<Item>, &'static str> {
    let character = client.character_id.ok_or("nobody to remember")?;
    let shop = shop_of(client, world, with).ok_or("no such keeper here")?;
    let visit = client.visit.as_ref().ok_or("not in a site")?;
    if !shop.in_reach(visit.position) {
        return Err("too far away");
    }
    let item = client
        .inventory
        .hand_over(item)
        .ok_or("can't hand that over")?;
    let level = world
        .site_mut(shop.id)
        .levels
        .entry(shop.index)
        .or_default();
    let record = level.records.entry(with).or_default();
    record.memory.remember_gift(character);
    record.carried.push(item);
    Ok(record.carried.clone())
}

/// What a keeper on the player's level has.
pub(crate) fn keeper_stock(
    client: &Client,
    world: &mut ServerWorld,
    with: Id,
) -> Option<Vec<Item>> {
    let shop = shop_of(client, world, with)?;
    let level = world.site(shop.id).levels.get(&shop.index)?;
    Some(level.records.get(&with)?.carried.clone())
}

/// Takes something from a shop's keeper, from beside its door, if it is still there
/// and can go where it is asked to. Free, until there is payment.
pub(crate) fn take_from_keeper(
    client: &mut Client,
    world: &mut ServerWorld,
    with: Id,
    item: Id,
    (to, x, y, rotation): (shared::item::Holder, u8, u8, shared::item::Rotation),
) -> Result<(), &'static str> {
    let shop = shop_of(client, world, with).ok_or("no such keeper here")?;
    let visit = client.visit.as_ref().ok_or("not in a site")?;
    if !shop.in_reach(visit.position) {
        return Err("too far away");
    }
    let level = world
        .site_mut(shop.id)
        .levels
        .entry(shop.index)
        .or_default();
    let carried = &mut level.records.entry(with).or_default().carried;
    let place = carried.iter().position(|one| one.id == item);
    let place = place.ok_or("they don't have that")?;
    let taken = carried[place].clone();
    client
        .inventory
        .add(taken, to, x, y, rotation)
        .map_err(|_| "it can't go there")?;
    carried.remove(place);
    Ok(())
}
