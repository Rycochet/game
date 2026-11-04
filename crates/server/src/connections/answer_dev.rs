//! Answering dev tools: teleports, sites made to order, items given, the clock and the like. `answer` only passes them on where dev tools are allowed.

use bevy::prelude::*;
use shared::item::Quality;
use shared::protocol::{ClientMessage, ServerMessage};
use shared::sheet::Sheet;

use super::{Client, made_for};
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
                let place = (visit.id(), visit.index());
                let stayed = sites::teleport(client, x, y);
                if let Some(visit) = &client.visit {
                    world.move_mob_aside(place, visit.position());
                }
                stayed
            }
            None => world_map::teleport(client, world, x, y),
        },
        ClientMessage::DevBackToStart => world_map::back_to_start(client, world),
        ClientMessage::DevToShowcase => world_map::to_showcase(client, world),
        ClientMessage::DevToHall => hall::to_hall(client, world),
        ClientMessage::DevEnterSite {
            kind,
            seed,
            level,
            ruin,
        } => sites::dev_enter(client, world, kind, seed, level, ruin),
        ClientMessage::DevGenerateSite { kind, seed, ruin } => {
            sites::generate(client, world, kind, seed, ruin)
        }
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
        } => {
            // Asking for magic on what is too plain for it would quietly give none.
            let quality = if magical && kind.def().wear.is_some() {
                quality.max(Quality::Uncommon)
            } else {
                quality
            };
            let mut new = made_for(
                &client.inventory,
                kind,
                size,
                magical,
                quality,
                world.made_now(),
            );
            new.set = set && new.quality.is_some();
            new.made.magical = magical && new.can_be_magical();
            new.favour = favour.filter(|_| kind.def().wear.is_some());
            match client.inventory.give(new, world.stacks) {
                Ok(()) => client.unsaved = true,
                Err(item) => info!("No room for a new {:?}", item.kind),
            }
            client.update_speed(world.action_clock.ticks());
            client.send_inventory()
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
        ClientMessage::DevSetCreature(kind) => {
            for item in client.inventory.change_body(kind, world.stacks) {
                info!("No room for {:?} on becoming {kind:?}: lost", item.kind);
            }
            client.sheet = Sheet::new(kind);
            client.unsaved = true;
            client
                .link
                .send(&ServerMessage::Sheet(client.sheet.clone()))
                && {
                    client.update_speed(world.action_clock.ticks());
                    client.send_inventory()
                }
        }
        ClientMessage::DevSnowLine(snow_line) => {
            world.snow_line = snow_line;
            client.link.send(&ServerMessage::SnowLine(snow_line))
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
