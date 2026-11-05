//! The world's time, snow line, pois and suns and moons.

use bevy::prelude::*;
use serde::Deserialize;
use server::debug_api::{BrpResult, Value, params, refused};
use shared::protocol::ClientMessage;

use super::done;
use crate::common::CurrentWorld;
use crate::connection::ToServer;

#[derive(Deserialize)]
pub(super) struct GoToWorldParams {
    seed: u64,
}

/// `client.go_to_world { "seed": 2 }`: asks the server to move the player to
/// that seed's world (Go to World).
pub(super) fn go_to_world(
    In(request): In<Option<Value>>,
    mut to_server: MessageWriter<ToServer>,
) -> BrpResult {
    let GoToWorldParams { seed } = params(request)?;
    to_server.write(ToServer(ClientMessage::DevGoToWorld { seed }));
    done()
}

#[derive(Deserialize)]
pub(super) struct SetTimeParams {
    hours: Option<f64>,
    minutes: Option<f64>,
}

/// `client.set_time { "hours": 12 }` (or `minutes`): moves the world's clock
/// to that long since the world began (`HOW_DEBUG_TIME`). Saved.
pub(super) fn set_time(
    In(request): In<Option<Value>>,
    mut to_server: MessageWriter<ToServer>,
) -> BrpResult {
    let SetTimeParams { hours, minutes } = params(request)?;
    let minutes = match (hours, minutes) {
        (_, Some(minutes)) => minutes,
        (Some(hours), None) => hours * shared::world::MINUTES_PER_HOUR,
        (None, None) => return Err(refused("give hours or minutes")),
    };
    to_server.write(ToServer(ClientMessage::DevSetWorldClock {
        minutes: Some(minutes),
        rate: None,
    }));
    done()
}

#[derive(Deserialize)]
pub(super) struct SnowLineParams {
    /// 128 is normal, 0 frozen.
    line: u32,
}

/// `client.snow_line { "line": 0 }` (`HOW_DEBUG_SNOW`).
pub(super) fn snow_line(
    In(request): In<Option<Value>>,
    mut to_server: MessageWriter<ToServer>,
) -> BrpResult {
    let SnowLineParams { line } = params(request)?;
    to_server.write(ToServer(ClientMessage::DevSnowLine(line)));
    done()
}

/// `client.reveal_pois`: every poi, on this connection (`HOW_DEBUG_REVEAL_POIS`).
pub(super) fn reveal_pois(
    _: In<Option<Value>>,
    mut to_server: MessageWriter<ToServer>,
) -> BrpResult {
    to_server.write(ToServer(ClientMessage::DevRevealPois));
    done()
}

#[derive(Deserialize)]
pub(super) struct SeedParams {
    seed: u64,
}

/// `client.celestial { "seed": 272 }`: the suns and moons of that terrain
/// seed, not this world's, until restarting (`HOW_DEBUG_CELESTIAL`).
pub(super) fn celestial(
    In(request): In<Option<Value>>,
    mut world: ResMut<CurrentWorld>,
) -> BrpResult {
    let SeedParams { seed } = params(request)?;
    world.celestial = shared::celestial::Celestial::from_seed(shared::rng::Seed::new(seed));
    done()
}
