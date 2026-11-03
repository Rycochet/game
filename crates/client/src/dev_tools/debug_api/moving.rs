//! Steps, travel and Auto Walk.

use bevy::prelude::*;
use serde::Deserialize;
use server::debug_api::{BrpResult, Value, params};
use shared::movement::steps_between;

use super::{Place, done};
use crate::debug_menu::AutoWalk;
use crate::player::{TakeStep, TravelTo};

#[derive(Deserialize)]
pub(super) struct Offset {
    #[serde(default)]
    dx: i64,
    #[serde(default)]
    dy: i64,
}

/// `client.step { "dx": 2, "dy": 0 }`: steps that far, as the keys do, so
/// not into walls (`HOW_DEBUG_MOVE`).
pub(super) fn step(
    In(request): In<Option<Value>>,
    mut steps: MessageWriter<TakeStep>,
) -> BrpResult {
    let Offset { dx, dy } = params(request)?;
    for step in steps_between((0, 0), (dx, dy)) {
        steps.write(TakeStep(step));
    }
    done()
}

/// `client.travel { "x": 5, "y": 3 }`: travels there as if clicked, on
/// either map (`HOW_DEBUG_TRAVEL`).
pub(super) fn travel(
    In(request): In<Option<Value>>,
    mut travel: MessageWriter<TravelTo>,
) -> BrpResult {
    let Place { x, y } = params(request)?;
    travel.write(TravelTo { x, y });
    done()
}

#[derive(Deserialize)]
pub(super) struct OnParams {
    #[serde(default = "yes")]
    on: bool,
}

pub(super) fn yes() -> bool {
    true
}

/// `client.walk { "on": true }`: Auto Walk, one step east per frame
/// (`HOW_DEBUG_WALK`).
pub(super) fn walk(In(request): In<Option<Value>>, mut walk: ResMut<AutoWalk>) -> BrpResult {
    let OnParams { on } = params(request)?;
    walk.0 = on;
    done()
}
