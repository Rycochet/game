//! Steps, travel and Auto Walk on starting.

use bevy::prelude::*;
use shared::movement::steps_between;
use std::env;

use super::parse_pos;
use crate::common::{GameView, MapPosition};
use crate::debug_menu::AutoWalk;
use crate::player::{Player, PlayerSystems, TakeStep, TravelTo};

/// `HOW_DEBUG_MOVE`, `HOW_DEBUG_TRAVEL` and `HOW_DEBUG_WALK`.
pub(super) fn add(app: &mut App) {
    if let Some((dx, dy)) = env::var("HOW_DEBUG_MOVE")
        .ok()
        .and_then(|pos| parse_pos(&pos))
    {
        app.add_systems(
            Update,
            (move |time: Res<Time>, mut done: Local<bool>, mut steps: MessageWriter<TakeStep>| {
                if !*done && time.elapsed_secs() > 2.0 {
                    *done = true;
                    for step in steps_between((0, 0), (dx, dy)) {
                        steps.write(TakeStep(step));
                    }
                }
            })
            .in_set(PlayerSystems::Input),
        );
    }

    if let Some((x, y)) = env::var("HOW_DEBUG_TRAVEL")
        .ok()
        .and_then(|pos| parse_pos(&pos))
    {
        let start = env::var("HOW_DEBUG_POS")
            .ok()
            .and_then(|pos| parse_pos(&pos));
        app.add_systems(
            Update,
            (move |time: Res<Time>,
                   mut done: Local<bool>,
                   player: Single<&MapPosition, With<Player>>,
                   mut travel: MessageWriter<TravelTo>| {
                // Not before the server has moved the player to `HOW_DEBUG_POS`
                // or back into their site: that would end the journey.
                let ready = match start {
                    Some((sx, sy)) => (player.x, player.y) == (sx, sy),
                    None => time.elapsed_secs() > 2.0,
                };
                if !*done && ready {
                    *done = true;
                    travel.write(TravelTo { x, y });
                }
            })
            .in_set(PlayerSystems::Input)
            .run_if(not(in_state(GameView::Connecting))),
        );
    }

    if server::option_on("HOW_DEBUG_WALK", false) {
        app.insert_resource(AutoWalk(true));
    }
}
