//! Give Item (dev tools) with a mob as the target: what it is, so things are made
//! to fit it, and putting them in its pockets, as a gift does.

use shared::creature::CreatureKind;
use shared::id::Id;
use shared::item::Item;

use super::gear::record_of;
use crate::ServerWorld;
use crate::connections::Client;

/// The kind of mob `id`, if it is on the player's level and in their sight.
pub(crate) fn mob_in_sight(
    client: &mut Client,
    world: &ServerWorld,
    id: Id,
) -> Option<CreatureKind> {
    let visit = client.visit.as_mut()?;
    let population = world.populations.0.get(&visit.population())?;
    let mob = population.mobs.iter().find(|mob| mob.id == id)?;
    let (x, y) = mob.position;
    visit.can_see(x, y).then_some(mob.kind)
}

/// Puts `items` with what mob `id` carries, kept with its level as a gift is. It
/// carries whatever it is given, as it does a gift.
pub(crate) fn give_to_mob(
    client: &Client,
    world: &mut ServerWorld,
    id: Id,
    items: Vec<Item>,
) -> Result<(), &'static str> {
    let visit = client.visit.as_ref().ok_or("not in a site")?;
    let place = visit.population();
    let population = world
        .populations
        .0
        .get(&place)
        .ok_or("nobody on this level")?;
    let mob = population
        .mobs
        .iter()
        .find(|mob| mob.id == id)
        .ok_or("no such mob here")?;
    let (home_level, home) = (mob.home_level, mob.home);
    record_of(world, place.0, (id, home_level, home))
        .carried
        .extend(items);
    Ok(())
}
