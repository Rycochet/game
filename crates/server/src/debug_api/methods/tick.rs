//! `server.tick`: holding the action clock still and moving it on a tick at a
//! time, to watch fights and mobs step by step.

use bevy::prelude::*;
use serde::Deserialize;

use super::super::{BrpResult, Value, json, params};
use crate::ServerWorlds;
use crate::clock::TickHold;

#[derive(Deserialize)]
pub(super) struct TickParams {
    /// Hold the clock (`true`) or let it run again (`false`); left as it is if
    /// missing, unless `count` is given, which holds it.
    hold: Option<bool>,
    /// Ticks to move on by, one a frame, while held.
    count: Option<u64>,
}

/// `server.tick { "count": 5 }` / `{ "hold": false }`: holds the action clock
/// and moves it on by `count` ticks, or lets it run with real time again.
/// Answers the ticks now, whether held, and the steps still to come.
pub(super) fn tick(
    In(request): In<Option<Value>>,
    mut hold: ResMut<TickHold>,
    worlds: Res<ServerWorlds>,
) -> BrpResult {
    let TickParams { hold: held, count } = params(request)?;
    if let Some(count) = count {
        hold.held = true;
        hold.steps += count;
    }
    if let Some(held) = held {
        hold.held = held;
        if !held {
            hold.steps = 0;
        }
    }
    Ok(json!({
        "ticks": worlds.action_clock().ticks(),
        "held": hold.held,
        "steps": hold.steps,
    }))
}
