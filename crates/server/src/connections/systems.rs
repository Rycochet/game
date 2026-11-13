//! The systems over every connection: saving, mobs, entities, the ground and the
//! time, each world with the players in it.

use bevy::prelude::*;
use shared::protocol::ServerMessage;
use shared::world::{MINUTES_PER_HOUR, WorldId};
use std::collections::BTreeSet;

use super::{Client, Connections, Phase};
use crate::ground::{self};
use crate::{ServerWorld, ServerWorlds};

/// Calls `each` with every open world and the clients in it (none for a world
/// nobody is in, which still has to settle its mobs). Clients are put in order
/// of their worlds for it, keeping their order within each.
pub(super) fn for_each_world(
    clients: &mut [Client],
    worlds: &mut ServerWorlds,
    mut each: impl FnMut(&mut [Client], &mut ServerWorld),
) {
    clients.sort_by_key(|client| client.world);
    for (id, world) in worlds.iter_mut() {
        let start = clients.partition_point(|client| client.world < Some(id));
        let end = clients.partition_point(|client| client.world <= Some(id));
        each(&mut clients[start..end], world);
    }
}

/// Saves every character and site that has changed, then closes worlds nobody
/// has been in for a while.
pub(crate) fn save_changes(mut connections: ResMut<Connections>, mut worlds: ResMut<ServerWorlds>) {
    save_all(&mut connections, &mut worlds);
    let in_use: BTreeSet<WorldId> = connections
        .clients
        .iter()
        .filter_map(|client| client.world)
        .collect();
    worlds.close_idle(&in_use);
}

/// Queues everything changed to be saved: mobs, characters and sites, in every
/// world, and the server's clocks.
pub(crate) fn save_all(connections: &mut Connections, worlds: &mut ServerWorlds) {
    for_each_world(&mut connections.clients, worlds, |clients, world| {
        world.save_mobs();
        for client in clients {
            client.save(world);
        }
        world.save_sites();
    });
    worlds.save_server();
}

/// Every tick: mobs do what they are doing.
pub(crate) fn run_mobs(
    mut connections: ResMut<Connections>,
    mut worlds: ResMut<ServerWorlds>,
    options: Res<crate::ServerOptions>,
) {
    for_each_world(&mut connections.clients, &mut worlds, |clients, world| {
        crate::entities::keep_wild_areas(clients, world);
        crate::entities::run_mobs(clients, world);
    });
    // Killed on Ironman: deleted, and asked for a new character.
    let Connections {
        clients, accounts, ..
    } = &mut *connections;
    for client in clients.iter_mut().filter(|client| client.died_for_good) {
        let world = worlds.of(client.world);
        let why = "died on Ironman";
        super::delete_character::delete_character(
            client,
            world,
            accounts,
            options.difficulties,
            why,
        );
    }
}

/// Every tick: tells everyone on a level whose doors have changed, and its mobs.
pub(crate) fn share_doors(mut connections: ResMut<Connections>, mut worlds: ResMut<ServerWorlds>) {
    for_each_world(
        &mut connections.clients,
        &mut worlds,
        crate::sites::share_doors,
    );
}

/// Every tick: tells everyone who has come into or gone out of their sight, or
/// moved. Who is on a level stays while it is in use. Everyone in the Hall can
/// see each other, whichever world they came from.
pub(crate) fn sync_entities(
    mut connections: ResMut<Connections>,
    mut worlds: ResMut<ServerWorlds>,
) {
    let in_hall = crate::entities::in_hall(&mut connections.clients);
    for_each_world(&mut connections.clients, &mut worlds, |clients, world| {
        crate::entities::sync(clients, world, &in_hall);
    });
}

/// Every tick: follows everyone for quests (their log, arriving at places,
/// the markers of the town they are in).
pub(crate) fn follow_quests(
    mut connections: ResMut<Connections>,
    mut worlds: ResMut<ServerWorlds>,
) {
    for_each_world(&mut connections.clients, &mut worlds, crate::quests::follow);
}

/// Every tick: tells everyone what has come to be on the ground under them.
pub(crate) fn sync_ground(mut connections: ResMut<Connections>, mut worlds: ResMut<ServerWorlds>) {
    for_each_world(&mut connections.clients, &mut worlds, ground::sync);
}

/// Every tick: tells everyone playing in each world if its time was moved, or has
/// reached a new whole hour (the tide follows it). A client that has gone is
/// noticed when its messages are next read.
pub(crate) fn tell_the_time(
    mut connections: ResMut<Connections>,
    mut worlds: ResMut<ServerWorlds>,
) {
    for_each_world(&mut connections.clients, &mut worlds, |clients, world| {
        let hour = (world.minutes() / MINUTES_PER_HOUR) as u32;
        let moved = std::mem::take(&mut world.clock_moved);
        let new_hour = hour != world.hour;
        world.hour = hour;
        let playing = |client: &&mut Client| client.phase == Phase::Playing;
        for client in clients.iter_mut().filter(playing) {
            if moved {
                client.link.send(&ServerMessage::WorldClock(world.time()));
            }
            if new_hour {
                client.link.send(&ServerMessage::Hour(hour));
            }
        }
    });
}
