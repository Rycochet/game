//! Developer tools for mobs (`DevMobTool`): one made to order on a cell of the
//! player's level (Create Mob; never kept), looking into one, deleting one, and
//! freezing them all.

use shared::dungeon::{Cell, Level};
use shared::id::Id;
use shared::mob::chosen_spawns;
use shared::protocol::{DevMob, DevMobTool, ServerMessage};
use shared::rng::Seed;

use super::made::DEV_MOB_NUMBERS;
use super::{Mob, dressed};
use crate::ServerWorld;
use crate::connections::Client;
use crate::ids::new_id;

/// Does what was asked. Whether the client is still welcome.
pub(crate) fn dev_mobs(client: &mut Client, world: &mut ServerWorld, tool: DevMobTool) -> bool {
    let done = match tool {
        DevMobTool::Create(order) => create_mob(client, world, order),
        DevMobTool::Inspect(mob) => return inspect(client, world, mob),
        DevMobTool::Delete(mob) => delete(client, world, mob),
        DevMobTool::Freeze(on) => {
            world.mobs_frozen = on;
            Ok(())
        }
    };
    if let Err(why) = done {
        bevy::log::info!("Refused {tool:?}: {why}");
    }
    true
}

/// The mob's population and level index, if it is on the player's level and in
/// their sight.
fn in_sight(
    client: &mut Client,
    world: &ServerWorld,
    mob: Id,
) -> Option<((crate::storage::SiteId, u32), u32)> {
    let visit = client.visit.as_mut()?;
    let place = visit.population();
    let population = world.populations.0.get(&place)?;
    let at = population.mobs.iter().find(|one| one.id == mob)?.position;
    visit.can_see(at.0, at.1).then_some((place, visit.index()))
}

/// Takes a mob in sight away for good, leaving nothing.
fn delete(client: &mut Client, world: &mut ServerWorld, mob: Id) -> Result<(), &'static str> {
    let (place, index) = in_sight(client, world, mob).ok_or("no such mob in sight")?;
    super::gear::bury(world, place, index, mob, false);
    Ok(())
}

/// Tells the client everything the server knows of a mob in sight.
fn inspect(client: &mut Client, world: &mut ServerWorld, mob: Id) -> bool {
    let Some((place, _)) = in_sight(client, world, mob) else {
        bevy::log::info!("Refused to inspect {mob:?}: not in sight");
        return true;
    };
    let tick = world.action_clock.ticks();
    let kind = client.inventory.creature;
    let character = client.character_id;
    let Some(one) = world.populations.0[&place]
        .mobs
        .iter()
        .find(|one| one.id == mob)
    else {
        return true;
    };
    let lines = super::inspect::describe(one, kind, character.as_ref(), tick, world.mobs_frozen);
    client.link.send(&ServerMessage::DevMobInfo { mob, lines })
}

/// Puts the mob on its cell of the player's site level, if it can stand there
/// and nobody is there. Not in the wilderness, whose mobs belong to its blocks.
fn create_mob(
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
