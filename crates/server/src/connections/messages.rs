//! Reading each client's messages, acting on queued steps, and joining a world.

use bevy::prelude::*;
use shared::energy::{CHANGE_MAP_COST, site_step_cost};
use shared::movement::Step;
use shared::protocol::ServerMessage;
use shared::role::Permission;
use shared::world::WorldId;

use super::look_up::answer_look_ups;
use super::moving::change_world;
use super::playtime::shows_activity;
use super::{Action, Client, Connections, Phase, answer, welcome};
use crate::accounts::Accounts;
use crate::ground::{self};
use crate::oidc::Logins;
use crate::sites::{self, Visit};
use crate::{ServerOptions, ServerWorld, ServerWorlds, gates, hall, world_map};

mod sneaking;
mod training;

pub(crate) use sneaking::Sneak;

/// Answers every message that has arrived, and forgets clients that have
/// gone (or were refused), saving them first.
pub(crate) fn handle_messages(
    mut connections: ResMut<Connections>,
    mut worlds: ResMut<ServerWorlds>,
    options: Res<ServerOptions>,
    logins: Option<Res<Logins>>,
) {
    let Connections { clients, accounts } = &mut *connections;
    for i in 0..clients.len() {
        let client = &mut clients[i];
        if client.gone {
            continue;
        }
        let world = worlds.of(client.world);
        let stay = loop {
            match client.link.receive() {
                Ok(Some(message)) => {
                    if shows_activity(&message) {
                        client.activity.did_something(world.action_clock.ticks());
                    }
                    if !answer(
                        client,
                        message,
                        world,
                        accounts,
                        &options,
                        logins.as_deref(),
                    ) {
                        break false;
                    }
                    // The rest once it has been welcomed (below).
                    if client.phase == Phase::Joining {
                        break true;
                    }
                }
                Ok(None) => {
                    break super::combat::land_ability(client, world)
                        && act(client, world)
                        && client.send_position()
                        && (client.phase != Phase::Playing || {
                            let now = world.action_clock.ticks();
                            client.keep_time(now, options.idle_minutes) && client.keep_vitals(now)
                        });
                }
                Err(error) => {
                    info!("A client left ({error:?})");
                    break false;
                }
            }
        };
        let stay = stay
            && match client.going_to.take() {
                Some(going) => change_world(client, &mut worlds, going, &options),
                None => true,
            };
        if !stay {
            client.disconnect(worlds.of(client.world));
        } else if client.phase == Phase::Joining
            && !join(clients, i, &mut worlds, accounts, &options)
        {
            let world = clients[i].world;
            clients[i].disconnect(worlds.of(world));
        }
        let world = clients[i].world;
        put_out_of_forgotten_sites(clients, i, worlds.of(world));
    }
    super::heal::heal_players(clients, &mut worlds);
    super::for_others::do_for_others(clients, &mut worlds);
    answer_look_ups(clients, &mut worlds);
    super::role_ask::refresh_roles(clients, accounts, &mut worlds);
    clients.retain(|client| !client.gone);
}

/// Does what a client has asked, as far as it has the energy, in order. `false` if gone.
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
            Action::EnterGate { entrance } => (gates::enter_gate(client, world, entrance), 0),
            Action::UseExit | Action::Down | Action::Up if client.sheet.vitals.fighting(tick) => {
                info!("Refused changing floors while fighting");
                (true, 0)
            }
            Action::UseExit => (sites::use_exit(client, world), 0),
            Action::Down => (sites::go_down(client, world), 0),
            Action::Up => (sites::go_up(client, world), 0),
            Action::OpenTrade { with } => (sites::open_trade(client, world, with), 0),
            Action::Ground(action) => ground::act(client, world, action),
            Action::Door(ask) => sites::door_act(client, world, ask),
            Action::UseAbility { ability, target } => {
                super::combat::start_ability(client, world, ability, target)
            }
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
    if let Some(from) = client.character.hall {
        let stayed = hall::stepped(client, step);
        let to = client.character.hall.unwrap_or(from);
        return (
            stayed,
            if to == from {
                0
            } else {
                site_step_cost(from, to)
            },
        );
    }
    match &client.visit {
        Some(visit) => {
            let from = visit.position();
            let place = visit.population();
            let wild = visit.is_wilderness();
            let gait = shared::movement::Gait::of(&client.sheet.skills);
            let landing = visit.landing(step, gait);
            let landed_on = landing.map(|at| visit.cells().cell(at.0, at.1));
            // Someone who won't move aside is in the way; anyone else swaps places.
            if let Some(to) = landing {
                let kind = client.inventory.creature;
                let character = client.character_id.as_ref();
                if !client.god_mode && world.enemy_at(place, to, kind, character) {
                    info!("Refused a step onto an enemy, at {to:?}");
                    return (true, 0);
                }
            }
            let stayed = sites::stepped(client, step) && sites::follow(client, world);
            let to = client.visit.as_ref().map_or(from, Visit::position);
            if to == from {
                return (stayed, 0);
            }
            world.swap_mob(place, to, from);
            // Where it led before any wrapping round the planet.
            let landed = landing.unwrap_or(to);
            let (moved, cost) = client.level_step(world, (from, landed), wild, landed_on);
            (stayed && moved, cost)
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
            let (moved, cost) = client.world_step(world, (from, to), ends);
            (stayed && moved, cost)
        }
    }
}

/// Puts anyone but client `i` (who did the forgetting, and has been dealt with)
/// out of any site of its world just forgotten: what they are on is no longer
/// that site.
pub(super) fn put_out_of_forgotten_sites(
    clients: &mut [Client],
    i: usize,
    world: &mut ServerWorld,
) {
    let forgotten = world.take_forgotten();
    if forgotten.is_empty() {
        return;
    }
    let clients_world = clients[i].world;
    for (j, other) in clients.iter_mut().enumerate() {
        let inside = other.world == clients_world
            && other
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
    worlds: &mut ServerWorlds,
    accounts: &mut Accounts,
    options: &ServerOptions,
) -> bool {
    let account = clients[i].account;
    let character = clients[i].character_id;
    let playing = clients
        .iter()
        .filter(|other| !other.gone && other.phase == Phase::Playing)
        .count();
    if options.player_limit > 0
        && !clients[i].may(Permission::PastPlayerLimit)
        && playing >= options.player_limit.into()
    {
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
            let who = account.map_or_else(|| other.in_log(), |id| format!("Account {}", id.0));
            info!("{who} connected again: dropping the older connection");
            other.link.send(&ServerMessage::Refused(
                "this account is playing somewhere else now".into(),
            ));
            other.disconnect(worlds.of(other.world));
        }
    }
    if let (Some(account), Some(character)) = (&account, &character)
        && let Err(error) = accounts.played(account, character)
    {
        return clients[i].refuse_for_saves(&error);
    }
    if let Some(character) = &character {
        clients[i].name = accounts.character_name(character).unwrap_or_default();
    }
    let Some(world) = world_for(&mut clients[i], worlds) else {
        return false;
    };
    clients[i].world = Some(world);
    welcome(&mut clients[i], worlds.of(Some(world)), options)
}

/// The world a joining character is in: the one their save says, opened if
/// need be, or the home for someone new or whose world can't be run (which is
/// logged). `None` if saves can't be reached, when the client is refused.
fn world_for(client: &mut Client, worlds: &mut ServerWorlds) -> Option<WorldId> {
    let id = client.character_id.expect("decided before joining");
    let saved = worlds.home().storage.character_world(&id);
    let found = saved.and_then(|world| match world {
        Some(world) => worlds.open_saved(world),
        None => Ok(None),
    });
    match found {
        Ok(found) => Some(found.unwrap_or(worlds.home_id())),
        Err(error) => {
            client.refuse_for_saves(&error);
            None
        }
    }
}
