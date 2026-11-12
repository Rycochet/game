//! Taking a player's steps on the world map and moves on levels and in the
//! Hall: whether each is allowed, and what it costs.

use bevy::prelude::*;
use shared::clearance::crowd_per_mille;
use shared::energy::site_move_cost;
use shared::movement::{Gait, Step};
use shared::spot::{CELL_SPAN, Spot};

use crate::connections::Client;
use crate::sites::{self, Visit};
use crate::{ServerWorld, hall, world_map};

/// Takes a world map step, if it is allowed: whether the client is still
/// there, and what it cost, which is nothing if it was refused. Steps
/// anywhere else are refused (`Position` puts the client right).
pub(super) fn step_taken(client: &mut Client, world: &mut ServerWorld, step: Step) -> (bool, u32) {
    if client.visit.is_some() || client.character.hall.is_some() {
        info!("Refused a world map step off the world map");
        return (true, 0);
    }
    let from = client.world_position();
    let stayed = world_map::stepped(client, world, step);
    if client.world_position() == from {
        return (stayed, 0);
    }
    // Where it led before any wrapping round the planet.
    let to = step.from(from);
    let conditions = world.conditions();
    let ends = (
        world.ground.elevation(from.0, from.1, conditions),
        world.ground.elevation(to.0, to.1, conditions),
    );
    let (moved, cost) = client.world_step(world, (from, to), ends);
    (stayed && moved, cost)
}

/// Takes a move to `to` (units, as the client counts them) on a level or in
/// the Hall, if it is allowed: whether the client is still there, and what it
/// cost, which is nothing if it was refused.
pub(super) fn move_taken(client: &mut Client, world: &mut ServerWorld, to: Spot) -> (bool, u32) {
    if client.visit.is_none() {
        let Some(from) = client.character.hall else {
            info!("Refused a move on the world map");
            return (true, 0);
        };
        let to = to.offset(client.hall_origin * CELL_SPAN, 0);
        let gait = Gait::of(&client.sheet.skills);
        let squeeze = hall::landing(from, to, gait, client.size())
            .map_or(1000, |landing| landing.squeeze_per_mille());
        if !hall::moved(client, to) {
            return (true, 0);
        }
        let cost = u64::from(site_move_cost(from, to)) * u64::from(squeeze) / 1000;
        return (true, cost as u32);
    }
    let Some(visit) = &client.visit else {
        return (true, 0);
    };
    let (from, from_cell) = (visit.spot(), visit.position());
    let to_cell = to.cell();
    let place = visit.population();
    let wild = visit.is_wilderness();
    let gait = Gait::of(&client.sheet.skills);
    let size = client.size();
    let Some(landing) = visit.landing(to, gait, size) else {
        return (sites::refused(client, to), 0);
    };
    let landed_on = visit.cells().cell(to_cell.0, to_cell.1);
    // Enemies are in the way; anyone else is squeezed past, slowly.
    let kind = client.inventory.creature;
    let character = client.character_id.as_ref();
    let others = match client.god_mode {
        true => Vec::new(),
        false => world.crowd_round(place, kind, character),
    };
    let Some(crowd) = crowd_per_mille(size, from, to, &others) else {
        info!("Refused a move into an enemy, to {to:?}");
        return (true, 0);
    };
    let squeeze = landing.squeeze_per_mille().max(crowd);
    let slower = |cost: u64| cost * u64::from(squeeze) / 1000;
    let stayed = sites::moved(client, to) && sites::follow(client, world);
    let Some(now) = client.visit.as_ref().map(Visit::spot) else {
        // Through a World Gate: the map change is charged instead.
        return (stayed, 0);
    };
    if now == from {
        return (stayed, 0);
    }
    if to_cell == from_cell {
        return (stayed, slower(site_move_cost(from, to).into()) as u32);
    }
    let (moved, cost) = client.level_step(world, (from_cell, to_cell), wild, Some(landed_on));
    // A step between two cells' middles costs what it always did; any other
    // move between those cells in proportion to its length.
    let between = Spot::middle_of(from_cell)
        .distance(Spot::middle_of(to_cell))
        .max(1);
    let cost = slower(u64::from(cost) * from.distance(to) / between);
    (stayed && moved, cost as u32)
}
