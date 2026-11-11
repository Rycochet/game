//! The wait before the player can act again, drawn over the action bar's
//! abilities: each slot's shade (`dom::SlotPart::Shade`) drains away from the
//! top as the wait ends. Every
//! ability waits alike, as they share the player's energy.

use bevy::prelude::*;

use crate::common::OwnAbility;
use crate::dom::SlotPart;

/// How much of the wait is left, from 1 when it began to 0 when it is over.
pub(super) fn wait_left(own: &OwnAbility, now: f64) -> f32 {
    let whole = own.ready - own.started;
    if whole <= 0.0 || now >= own.ready {
        return 0.0;
    }
    ((own.ready - now) / whole).clamp(0.0, 1.0) as f32
}

/// Sizes every shade to the wait left.
pub(super) fn show_wait(
    own: Res<OwnAbility>,
    time: Res<Time>,
    mut shades: Query<(&mut Node, &SlotPart)>,
) {
    let left = wait_left(&own, time.elapsed_secs_f64());
    for (mut node, part) in &mut shades {
        if !matches!(part, SlotPart::Shade(_)) {
            continue;
        }
        let height = percent(left * 100.0);
        if node.height != height {
            node.height = height;
        }
    }
}
