//! The systems over every connection: saving, mobs, entities, the ground and the time.

use bevy::prelude::*;
use shared::protocol::ServerMessage;
use shared::world::MINUTES_PER_HOUR;

use super::{Client, Connections, Phase};
use crate::ServerWorld;
use crate::ground::{self};

/// Saves every character and site that has changed.
pub(crate) fn save_changes(mut connections: ResMut<Connections>, mut world: ResMut<ServerWorld>) {
    save_all(&mut connections, &mut world);
}

/// Queues everything changed to be saved: mobs, characters and sites.
pub(crate) fn save_all(connections: &mut Connections, world: &mut ServerWorld) {
    world.save_mobs();
    for client in &mut connections.clients {
        client.save(world);
    }
    world.save_sites();
}

/// Every tick: mobs do what they are doing.
pub(crate) fn run_mobs(mut connections: ResMut<Connections>, mut world: ResMut<ServerWorld>) {
    crate::entities::run_mobs(&mut connections.clients, &mut world);
}

/// Every tick: tells everyone who has come into or gone out of their sight, or
/// moved. Who is on a level stays while it is in use.
pub(crate) fn sync_entities(mut connections: ResMut<Connections>, mut world: ResMut<ServerWorld>) {
    crate::entities::sync(&mut connections.clients, &mut world);
}

/// Every tick: tells everyone what has come to be on the ground under them.
pub(crate) fn sync_ground(mut connections: ResMut<Connections>, mut world: ResMut<ServerWorld>) {
    ground::sync(&mut connections.clients, &mut world);
}

/// Every tick: tells everyone playing if the clock was moved, or has reached a
/// new whole hour (the tide follows it). A client that has gone is noticed when
/// its messages are next read.
pub(crate) fn tell_the_time(connections: Res<Connections>, mut world: ResMut<ServerWorld>) {
    let minutes = world.clock.minutes();
    let hour = (minutes / MINUTES_PER_HOUR) as u32;
    let moved = std::mem::take(&mut world.clock_moved);
    let new_hour = hour != world.hour;
    world.hour = hour;
    let playing = |client: &&Client| client.phase == Phase::Playing;
    for client in connections.clients.iter().filter(playing) {
        if moved {
            client.link.send(&ServerMessage::Clock(minutes));
        }
        if new_hour {
            client.link.send(&ServerMessage::Hour(hour));
        }
    }
}
