//! Reading each client's messages, acting on queued steps, and joining a world.

use bevy::prelude::*;
use shared::energy::{CHANGE_MAP_COST, site_step_cost};
use shared::movement::{Step, world_step_cost};
use shared::protocol::ServerMessage;

use super::{Action, Client, Connections, Phase, answer, welcome};
use crate::accounts::Accounts;
use crate::ground::{self};
use crate::sites::{self, Visit};
use crate::{ServerOptions, ServerWorld, world_map};

/// Answers every message that has arrived, and forgets clients that have
/// gone (or were refused), saving them first.
pub(crate) fn handle_messages(
    mut connections: ResMut<Connections>,
    mut world: ResMut<ServerWorld>,
    options: Res<ServerOptions>,
) {
    let Connections { clients, accounts } = &mut *connections;
    for i in 0..clients.len() {
        let client = &mut clients[i];
        if client.gone {
            continue;
        }
        let stay = loop {
            match client.link.receive() {
                Ok(Some(message)) => {
                    if !answer(client, message, &mut world, accounts, &options) {
                        break false;
                    }
                    // The rest once it has been welcomed (below).
                    if client.phase == Phase::Joining {
                        break true;
                    }
                }
                Ok(None) => break act(client, &mut world) && client.send_position(),
                Err(error) => {
                    info!("A client left ({error:?})");
                    break false;
                }
            }
        };
        if !stay {
            client.disconnect(&mut world);
        } else if client.phase == Phase::Joining
            && !join(clients, i, &mut world, accounts, &options)
        {
            clients[i].disconnect(&mut world);
        }
        put_out_of_forgotten_sites(clients, i, &mut world);
    }
    clients.retain(|client| !client.gone);
}

/// Does what a client has asked, as far as it has the energy, in order. `false` if
/// the client has gone.
pub(super) fn act(client: &mut Client, world: &mut ServerWorld) -> bool {
    let tick = world.action_clock.ticks();
    while client.energy.ready(tick, client.speed) {
        let Some(action) = client.waiting.pop_front() else {
            break;
        };
        let map = client.map;
        let (stayed, cost) = match action {
            Action::Step { number, step } => {
                client.last_step = Some(number);
                step_taken(client, world, step)
            }
            Action::EnterSite { chunk } => (sites::enter_poi(client, world, chunk), 0),
            Action::UseExit => (sites::use_exit(client, world), 0),
            Action::OpenTrade { with } => (sites::open_trade(client, world, with), 0),
            Action::Ground(action) => ground::act(client, world, action),
        };
        if !stayed {
            return false;
        }
        let cost = if client.map == map {
            cost
        } else {
            CHANGE_MAP_COST
        };
        client.energy.spend(tick, client.speed, cost);
    }
    true
}

/// Takes a step, if it is allowed: whether the client is still there, and what it
/// cost, which is nothing if it was refused.
pub(super) fn step_taken(client: &mut Client, world: &mut ServerWorld, step: Step) -> (bool, u32) {
    match &client.visit {
        Some(visit) => {
            let from = visit.position();
            let place = (visit.id(), visit.index());
            // Someone who won't move aside is in the way; anyone else swaps places.
            if let Some(to) = visit.landing(step) {
                let kind = client.inventory.creature;
                let character = client.character_id.as_ref();
                if !client.god_mode && world.enemy_at(place, to, kind, character) {
                    info!("Refused a step onto an enemy, at {to:?}");
                    return (true, 0);
                }
            }
            let stayed = sites::stepped(client, step);
            let to = client.visit.as_ref().map_or(from, Visit::position);
            if to == from {
                return (stayed, 0);
            }
            world.swap_mob(place, to, from);
            (stayed, site_step_cost(from, to))
        }
        None => {
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
            (stayed, world_step_cost(from, to, ends))
        }
    }
}

/// Puts anyone but client `i` (who did the forgetting, and has been dealt with)
/// out of any site just forgotten: what they are on is no longer that site.
pub(super) fn put_out_of_forgotten_sites(
    clients: &mut [Client],
    i: usize,
    world: &mut ServerWorld,
) {
    let forgotten = world.take_forgotten();
    if forgotten.is_empty() {
        return;
    }
    for (j, other) in clients.iter_mut().enumerate() {
        let inside = other
            .visit
            .as_ref()
            .is_some_and(|visit| forgotten.includes(visit.id()));
        if j != i && !other.gone && inside && !sites::put_out(other, world) {
            other.disconnect(world);
        }
    }
}

/// Welcomes client `i`, whose character is decided. Anyone else playing that
/// account or character is disconnected first (and saved, so the character
/// loads as they left it): the newest connection wins, for coming back after a
/// connection drops before the server has noticed. One character per account
/// is online at a time. `false` if the client has gone.
pub(super) fn join(
    clients: &mut [Client],
    i: usize,
    world: &mut ServerWorld,
    accounts: &mut Accounts,
    options: &ServerOptions,
) -> bool {
    let account = clients[i].account;
    let character = clients[i].character_id;
    let playing = clients
        .iter()
        .filter(|other| !other.gone && other.phase == Phase::Playing)
        .count();
    if options.player_limit > 0 && !clients[i].host && playing >= options.player_limit.into() {
        info!("A player was turned away: the server is full");
        clients[i].link.send(&ServerMessage::Refused(
            "this server is full: try again later".into(),
        ));
        return false;
    }
    for (j, other) in clients.iter_mut().enumerate() {
        let same_account = account.is_some() && other.account == account;
        let same = same_account || other.character_id == character;
        if j != i && !other.gone && other.phase == Phase::Playing && same {
            info!("{character:?} connected again: dropping the older connection");
            other.link.send(&ServerMessage::Refused(
                "this account is playing somewhere else now".into(),
            ));
            other.disconnect(world);
        }
    }
    if let (Some(account), Some(character)) = (&account, &character)
        && let Err(error) = accounts.played(account, character)
    {
        return clients[i].refuse_for_saves(&error);
    }
    welcome(&mut clients[i], world, options)
}
