//! Keeping the server's clock running and put right by pongs; single player's pings.

use bevy::prelude::*;
use shared::protocol::ClientMessage;

use super::ping::PING_EVERY;
use super::{ServerConnection, ToServer};
use crate::common::ServerClock;

/// How much of what the clock is out by is put right at each answer.
pub(super) const CLOCK_EASING: f64 = 0.1;

/// Seconds the server's clock can be out by before it jumps.
pub(super) const CLOCK_JUMP_SECONDS: f64 = 5.0;

/// The clock keeps running between the server's answers, and the world's
/// time with it.
pub(super) fn run_clock(time: Res<Time<Real>>, mut clock: ResMut<ServerClock>) {
    if !clock.paused {
        let (seconds, world) = (clock.seconds + time.delta_secs_f64(), clock.time);
        clock.set(seconds, world);
    }
}

/// Everything going by `Time` stops while the server's clock has: travel, and
/// clouds, water and rain moving. What goes by `Time<Real>`, as menus do, doesn't.
pub(super) fn stop_game_time(clock: Res<ServerClock>, mut time: ResMut<Time<Virtual>>) {
    if clock.paused != time.is_paused() {
        if clock.paused {
            time.pause();
        } else {
            time.unpause();
        }
    }
}

/// Puts the clock right from the server's latest answer. After `run_clock`, so
/// the clock has caught up with any long frame the answer waited through.
pub(super) fn apply_pong(
    connection: Option<Res<ServerConnection>>,
    mut clock: ResMut<ServerClock>,
) {
    let Some(pong) = connection.and_then(|connection| connection.pongs.take()) else {
        return;
    };
    clock.round_trip_ms = Some(pong.round_trip * 1000.0);
    // `Paused` said exactly where it stopped, and the time below hasn't passed.
    if clock.paused {
        return;
    }
    // The server read its clock about half the round trip before the answer
    // arrived, and the answer may have been waiting for this frame since.
    let ago = pong.round_trip / 2.0 + pong.received.elapsed().as_secs_f64();
    let behind = pong.clock + ago - clock.seconds;
    // Far out: jump. Otherwise ease, so clouds never visibly jerk.
    let seconds = clock.seconds
        + if behind.abs() > CLOCK_JUMP_SECONDS {
            behind
        } else {
            behind * CLOCK_EASING
        };
    let world = clock.time;
    clock.set(seconds, world);
}

/// Single player only, once in a world: asks the embedded server's clock now
/// and then. Over a network, the socket does this (`ping`).
pub(super) fn send_local_pings(
    time: Res<Time<Real>>,
    connection: Option<Res<ServerConnection>>,
    mut to_server: MessageWriter<ToServer>,
    mut last: Local<Option<f64>>,
) {
    if !connection.is_some_and(|connection| connection.local) {
        return;
    }
    let now = time.elapsed_secs_f64();
    if last.is_none_or(|last| now - last >= PING_EVERY.as_secs_f64()) {
        *last = Some(now);
        to_server.write(ToServer(ClientMessage::Ping { sent: now }));
    }
}
