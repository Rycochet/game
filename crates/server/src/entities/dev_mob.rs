//! Developer tools: a mob made to order on a cell of the player's level
//! (Create Mob). It is never kept: it goes when the level is let go.

use shared::dungeon::{Cell, Level};
use shared::mob::chosen_spawns;
use shared::protocol::DevMob;
use shared::rng::Seed;

use super::made::DEV_MOB_NUMBERS;
use super::{Mob, dressed};
use crate::ServerWorld;
use crate::connections::Client;
use crate::ids::new_id;

/// Puts the mob on its cell of the player's site level, if it can stand there
/// and nobody is there. Not in the wilderness, whose mobs belong to its blocks.
pub(crate) fn create_mob(
    client: &mut Client,
    world: &mut ServerWorld,
    order: DevMob,
) -> Result<(), &'static str> {
    let visit = client.visit.as_ref().ok_or("not in a site")?;
    if visit.is_wilderness() {
        return Err("not in the wilderness");
    }
    let (place, index, player) = (visit.population(), visit.index(), visit.position());
    let cell = (order.x, order.y);
    let made_at = world.made_now();
    let population = world
        .populations
        .0
        .get_mut(&place)
        .ok_or("this level isn't in use")?;
    if !population.level.is_walkable(cell.0, cell.1) {
        return Err("nothing can stand there");
    }
    if cell == player || population.mobs.iter().any(|mob| mob.position == cell) {
        return Err("someone is there");
    }
    // Rolled as a level's own, on a level of just that cell.
    let mut here = Level::new(cell.0, cell.1, 1, 1);
    here.set(cell.0, cell.1, Cell::Floor);
    let seed = Seed::new(order.seed).derive("dev mob");
    let mut spawn = chosen_spawns(seed, order.kind, 1, &here, &[])
        .pop()
        .ok_or("no mob could be rolled")?;
    if !order.gear {
        spawn.gear.clear();
    }
    let most = population.mobs.iter().map(|mob| mob.number).max();
    spawn.number = most
        .filter(|&n| n >= DEV_MOB_NUMBERS)
        .map_or(DEV_MOB_NUMBERS, |n| n + 1);
    let carrying = dressed(spawn.kind, &spawn.gear, made_at);
    let mut mob = Mob::new(new_id(), &spawn, &carrying, index);
    mob.still = order.still;
    if let (Some(standing), Some(character)) = (order.attitude, client.character_id) {
        mob.memory.set_standing(character, standing);
    }
    population.mobs.push(mob);
    Ok(())
}
