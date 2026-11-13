//! Finding and changing the bags on a level's ground: by where they are in
//! units, within someone's reach (`shared::ground::in_reach`).

use shared::creature::Size;
use shared::ground::in_reach;
use shared::id::Id;
use shared::item::Item;
use shared::spot::Spot;

use crate::ServerWorld;
use crate::connections::Client;
use crate::storage::{LevelSave, SiteId};

/// The bags on a level's ground: where each is, and what is in it, oldest first.
type Bags = crate::storage::SpotMap<Vec<Item>>;

/// Where the player stands, in units, and which level it is on.
pub(super) fn standing_at(client: &Client) -> Option<(SiteId, u32, Spot)> {
    let visit = client.visit.as_ref()?;
    Some((visit.id(), visit.index(), visit.spot()))
}

/// The level the player is on, to change.
fn level_mut<'a>(world: &'a mut ServerWorld, client: &Client) -> Option<&'a mut LevelSave> {
    let (id, index, _) = standing_at(client)?;
    Some(world.site_mut(id).levels.entry(index).or_default())
}

/// The bags someone of this size standing at `at` can reach, nearest first.
pub(super) fn bags_in_reach(bags: &Bags, size: Size, at: Spot) -> Vec<Spot> {
    let mut found: Vec<Spot> = (bags.keys().copied())
        .filter(|&bag| in_reach(size, at, bag))
        .collect();
    found.sort_by_key(|&bag| (at.distance_squared(bag), bag));
    found
}

/// Puts these on the ground for someone of this size at `at`: into the nearest
/// bag in reach, or a new one at their feet.
pub(crate) fn drop_at(
    bags: &mut Bags,
    size: Size,
    at: Spot,
    items: impl IntoIterator<Item = Item>,
) {
    let into = bags_in_reach(bags, size, at).first().copied().unwrap_or(at);
    bags.entry(into).or_default().extend(items);
}

/// Puts this on the ground by the player, as the newest in its bag.
pub(crate) fn put_down(client: &Client, world: &mut ServerWorld, mut item: Item) {
    // Quest items are destroyed rather than left for anyone (owner): one on
    // its own, or any inside what is put down.
    if item.is_quest_item() {
        return;
    }
    item.strip_quest_items();
    let size = client.size();
    if let Some((.., at)) = standing_at(client)
        && let Some(level) = level_mut(world, client)
    {
        drop_at(&mut level.ground, size, at, [item]);
    }
}

/// The bag in the player's reach this is in, nearest first.
fn holding(client: &Client, bags: &Bags, item: Id) -> Option<Spot> {
    let (.., at) = standing_at(client)?;
    let reachable = bags_in_reach(bags, client.size(), at);
    reachable
        .into_iter()
        .find(|bag| bags[bag].iter().any(|one| one.id == item))
}

/// Takes this out of its bag in the player's reach. A bag left empty goes.
pub(super) fn take_up(client: &Client, world: &mut ServerWorld, item: Id) -> Option<Item> {
    let level = level_mut(world, client)?;
    let bag = holding(client, &level.ground, item)?;
    let pile = level.ground.get_mut(&bag)?;
    let place = pile.iter().position(|one| one.id == item)?;
    let taken = pile.remove(place);
    if pile.is_empty() {
        level.ground.remove(&bag);
    }
    Some(taken)
}

/// What is left in its bag of something partly taken up: `None` if all of it was.
pub(super) fn leave_on_ground(
    client: &Client,
    world: &mut ServerWorld,
    item: Id,
    rest: Option<Item>,
) {
    let Some(rest) = rest else {
        take_up(client, world, item);
        return;
    };
    let Some(level) = level_mut(world, client) else {
        return;
    };
    let bag = holding(client, &level.ground, item);
    let pile = bag.and_then(|bag| level.ground.get_mut(&bag));
    if let Some(one) = pile.and_then(|pile| pile.iter_mut().find(|one| one.id == item)) {
        *one = rest;
    }
}

/// Something in a bag in the player's reach, as it is: it stays there until
/// `take_up`.
pub(super) fn lying(
    client: &Client,
    world: &mut ServerWorld,
    item: Id,
) -> Result<Item, &'static str> {
    let (id, index, at) = standing_at(client).ok_or("not in a site")?;
    if let Some(find) = own_find(client, world, item, at) {
        return Ok(find.item);
    }
    let level = world.site(id).levels.get(&index).ok_or("nothing here")?;
    let bag = holding(client, &level.ground, item).ok_or("that isn't in reach")?;
    let found = level.ground[&bag].iter().find(|one| one.id == item);
    found.cloned().ok_or("that isn't in reach")
}

/// The player's own quest item lying in reach, if `item` is it: it lies for
/// them alone (`quests::find_here`), so taking it takes nothing from a bag.
pub(super) fn own_find(
    client: &Client,
    world: &mut ServerWorld,
    item: Id,
    at: Spot,
) -> Option<crate::quests::Find> {
    let find = crate::quests::find_here(client, world)?;
    (find.item.id == item && in_reach(client.size(), at, find.at)).then_some(find)
}
