//! Eating from the ground: the top one of food in a bag in the player's reach.

use shared::bonus::is_food;
use shared::id::Id;
use shared::item::Item;

use super::bags::{leave_on_ground, lying};
use crate::ServerWorld;
use crate::connections::Client;
use crate::ids::new_id;

/// Takes the top one of this, if it is food lying in the player's reach, to be
/// eaten: the rest of its stack stays in its bag.
pub(crate) fn take_food(
    client: &Client,
    world: &mut ServerWorld,
    item: Id,
) -> Result<Item, &'static str> {
    let mut found = lying(client, world, item)?;
    if !is_food(found.top().kind) {
        return Err("it can't be eaten");
    }
    let top = found.take_top(&mut new_id);
    let rest = top.is_some().then(|| found.clone());
    leave_on_ground(client, world, item, rest);
    Ok(top.unwrap_or(found))
}
