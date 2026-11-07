//! Shopkeepers: who keeps which building, and giving and taking their stock.

use bevy::prelude::*;
use shared::id::Id;
use shared::item::Item;
use shared::shop::DOOR_REACH;

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

/// Which building on the player's level this is the keeper of, if any.
pub(crate) fn keeper_building(client: &Client, world: &mut ServerWorld, with: Id) -> Option<u16> {
    let visit = client.visit.as_ref()?;
    let level = world.site(visit.id).levels.get(&visit.index)?;
    let mut buildings = level.buildings.iter();
    buildings
        .find(|(_, save)| save.keeper == Some(with))
        .map(|(&building, _)| building)
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
    let building = keeper_building(client, world, with).ok_or("no such keeper here")?;
    let visit = client.visit.as_ref().ok_or("not in a site")?;
    let town = visit.level.buildings.iter().find(|b| b.look.id == building);
    let door = town.ok_or("no such building here")?.look.door;
    let (x, y) = visit.position;
    if (x - door.0).abs().max((y - door.1).abs()) > DOOR_REACH {
        return Err("too far away");
    }
    let (id, index) = (visit.id, visit.index);
    let item = client
        .inventory
        .hand_over(item)
        .ok_or("can't hand that over")?;
    let level = world.site_mut(id).levels.entry(index).or_default();
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
    keeper_building(client, world, with)?;
    let visit = client.visit.as_ref()?;
    let level = world.site(visit.id).levels.get(&visit.index)?;
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
    let building = keeper_building(client, world, with).ok_or("no such keeper here")?;
    let visit = client.visit.as_ref().ok_or("not in a site")?;
    let town = visit.level.buildings.iter().find(|b| b.look.id == building);
    let door = town.ok_or("no such building here")?.look.door;
    let (at_x, at_y) = visit.position;
    if (at_x - door.0).abs().max((at_y - door.1).abs()) > DOOR_REACH {
        return Err("too far away");
    }
    let (id, index) = (visit.id, visit.index);
    let level = world.site_mut(id).levels.entry(index).or_default();
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
