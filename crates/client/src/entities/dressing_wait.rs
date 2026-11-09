//! Changing clothes shown (a placeholder, owner 2026-11-09): how long the player's
//! last putting on or taking off keeps them waiting, for `wind_up_bars` to show as
//! the same gold bar under their token while it is a second or more. To be
//! replaced by a properly designed progress widget.

use bevy::prelude::*;
use shared::energy::ACTION_TICKS_PER_SECOND;

use crate::common::{Dressing, PlayerSpeed};

/// When the wait for changing clothes began and ends, in `Time` seconds. Changes
/// made while waiting add on to it, as the energy they cost does.
#[derive(Resource, Default)]
pub(super) struct DressingWait {
    pub(super) started: f64,
    pub(super) ends: f64,
}

pub(super) fn take_dressing(
    mut dressing: MessageReader<Dressing>,
    speed: Option<Res<PlayerSpeed>>,
    time: Res<Time>,
    mut wait: ResMut<DressingWait>,
) {
    let per_second = speed.map_or(1, |speed| speed.0.max(1)) as f64 * ACTION_TICKS_PER_SECOND;
    let now = time.elapsed_secs_f64();
    for &Dressing(cost) in dressing.read() {
        if cost == 0 {
            continue;
        }
        if wait.ends <= now {
            wait.started = now;
            wait.ends = now;
        }
        wait.ends += cost as f64 / per_second;
    }
}
