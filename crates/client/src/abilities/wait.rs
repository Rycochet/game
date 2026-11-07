//! The wait before the player can act again, drawn over the action bar's
//! abilities: a shade that drains away from the top as the wait ends. Every
//! ability waits alike, as they share the player's energy.

use bevy::prelude::*;

use crate::common::OwnAbility;
use crate::widgets::Floating;

/// How dark the shade is.
const SHADE: Color = Color::srgba(0.0, 0.0, 0.0, 0.6);

/// The shade over one slot's ability.
#[derive(Component)]
pub(super) struct WaitShade;

/// A shade for a slot holding an ability, empty until there is a wait.
pub(super) fn spawn_shade(inside: &mut ChildSpawnerCommands) {
    inside.spawn((
        WaitShade,
        Floating,
        Node {
            position_type: PositionType::Absolute,
            left: px(0),
            right: px(0),
            bottom: px(0),
            height: percent(0),
            border_radius: BorderRadius::all(px(4.0)),
            ..default()
        },
        BackgroundColor(SHADE),
    ));
}

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
    mut shades: Query<&mut Node, With<WaitShade>>,
) {
    let left = wait_left(&own, time.elapsed_secs_f64());
    for mut node in &mut shades {
        let height = percent(left * 100.0);
        if node.height != height {
            node.height = height;
        }
    }
}
