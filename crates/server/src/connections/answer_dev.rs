//! Answering dev tools: teleports, sites made to order, items given, the clock and the like. `answer` only passes them on where dev tools are allowed.

use bevy::prelude::*;
use shared::item_form::ItemOrder;
use shared::protocol::ClientMessage;

use super::Client;
use super::give_items::{Batch, give_items};
use crate::sites::{self};
use crate::{ServerWorld, hall, world_map};

/// Answers a dev tool. Whether the client is still welcome.
pub(super) fn answer_dev(
    client: &mut Client,
    message: ClientMessage,
    world: &mut ServerWorld,
) -> bool {
    match message {
        ClientMessage::DevTeleport { x, y }
            if client.visit.is_none() && client.character.hall.is_some() =>
        {
            hall::teleport(client, x, y)
        }
        ClientMessage::DevTeleport { x, y } => match &client.visit {
            Some(visit) => {
                let place = visit.population();
                let stayed = sites::teleport(client, x, y);
                if let Some(visit) = &client.visit {
                    world.move_mob_aside(place, visit.position());
                }
                stayed
            }
            None => world_map::teleport(client, world, x, y),
        },
        ClientMessage::DevBackToStart => world_map::back_to_start(client, world),
        ClientMessage::DevToHall { seed, gate } => hall::to_hall(client, world, seed, gate),
        // Told by `tell_of_lightning`, which knows where everyone is.
        ClientMessage::DevLightning => {
            world.lightning_asked.insert(client.entity_id());
            true
        }
        ClientMessage::DevVitals(what) => client.dev_vitals(what, world.action_clock.ticks()),
        ClientMessage::DevSkills { who, change } => {
            client.dev_skills(who, change, world.action_clock.ticks())
        }
        ClientMessage::DevEnterSite {
            kind,
            seed,
            level,
            ruin,
        } => sites::dev_enter(client, world, kind, seed, level, ruin),
        ClientMessage::DevGenerateSite { kind, seed, ruin } => {
            sites::generate(client, world, kind, seed, ruin)
        }
        ClientMessage::DevGenerateArea(area) => sites::generate_area(client, world, area),
        ClientMessage::DevMobs(tool) => crate::entities::dev_mobs(client, world, tool),
        ClientMessage::DevSetCell { x, y, cell } => sites::set_cell(client, world, x, y, cell),
        ClientMessage::DevRevealPois => world_map::reveal_pois(client, world),
        ClientMessage::DevRevealMap => sites::reveal_level(client, world),
        ClientMessage::DevForgetMap => match client.visit {
            Some(_) => sites::forget_level(client, world),
            None => world_map::forget(client, world),
        },
        ClientMessage::DevForgetMapOnServer => match client.visit {
            Some(_) => sites::forget_site(client, world),
            None => world_map::forget_on_server(client, world),
        },
        ClientMessage::DevGiveItem {
            kind,
            size,
            magical,
            quality,
            set,
            favour,
            seed,
            count,
            identical,
            request,
            to,
        } => {
            let order = ItemOrder {
                kind,
                size,
                magical,
                quality,
                set,
                favour,
                request,
            };
            let batch = Batch {
                order,
                seed,
                count,
                identical,
            };
            give_items(client, world, batch, to)
        }
        ClientMessage::DevDetectMonsters(on) => {
            client.detect_monsters = on;
            true
        }
        ClientMessage::DevTruesight(on) => {
            client.truesight = on;
            client.ground_told.again = true;
            client.send_inventory() && sites::truesight_changed(client, world)
        }
        ClientMessage::DevGodMode(on) => {
            client.god_mode = on;
            true
        }
        ClientMessage::DevKillMob { mob } => {
            if let Err(why) = crate::entities::kill(client, world, mob) {
                warn!("Refused to kill {mob:?}: {why}");
            }
            true
        }
        ClientMessage::DevSetCreature { kind, how } => {
            super::change_kind::change_kind(client, world, kind, how)
        }
        ClientMessage::DevGoToWorld { seed, gate } => {
            // Once its messages this tick have been read (`handle_messages`), as
            // it needs every world.
            let seed = seed.map_or(world.seed, shared::world::WorldSeed::new);
            let arrival = match gate {
                Some(gate) => super::Arrival::WorldGate(gate),
                None => super::Arrival::AsBefore,
            };
            client.go_to_world(seed, arrival);
            true
        }
        ClientMessage::DevSetWorldClock { minutes, rate } => {
            // Not trusted to be one: a rate that stops time is refused.
            let rate = rate.map(|rate| shared::world::ClockRate::new(rate.game, rate.real));
            if rate.is_some_and(|rate| rate.is_none()) {
                warn!("A client asked for a clock rate that stops time");
                return true;
            }
            world.set_time(minutes, rate.flatten());
            true
        }
        _ => unreachable!("`answer` only passes dev tools here"),
    }
}
