//! The world's time, suns and moons and snow line on starting.

use bevy::prelude::*;
use shared::protocol::ClientMessage;
use std::env;

use crate::common::GameView;
use crate::connection::ToServer;

/// `HOW_DEBUG_TIME`, `HOW_DEBUG_CELESTIAL` and `HOW_DEBUG_SNOW`.
pub(super) fn add(app: &mut App) {
    if let Some(hours) = env::var("HOW_DEBUG_TIME")
        .ok()
        .and_then(|v| v.parse::<f64>().ok())
    {
        // Asked of the server once there is one to ask.
        app.add_systems(
            Update,
            move |view: Res<State<GameView>>,
                  mut done: Local<bool>,
                  mut to_server: MessageWriter<ToServer>| {
                if !*done && *view.get() != GameView::Connecting {
                    *done = true;
                    to_server.write(ToServer(ClientMessage::DevSetTime {
                        minutes: hours * shared::world::MINUTES_PER_HOUR,
                    }));
                }
            },
        );
    }

    if let Some(seed) = env::var("HOW_DEBUG_CELESTIAL")
        .ok()
        .and_then(|v| v.parse().ok())
    {
        app.add_systems(
            OnExit(GameView::Connecting),
            move |mut world: ResMut<crate::common::CurrentWorld>| {
                let seed = shared::rng::Seed::new(seed);
                world.celestial = shared::celestial::Celestial::from_seed(seed);
            },
        );
    }

    if let Some(snow_line) = env::var("HOW_DEBUG_SNOW").ok().and_then(|v| v.parse().ok()) {
        // Asked of the server once there is one to ask.
        app.add_systems(
            Update,
            move |view: Res<State<GameView>>,
                  mut done: Local<bool>,
                  mut to_server: MessageWriter<ToServer>| {
                if !*done && *view.get() != GameView::Connecting {
                    *done = true;
                    to_server.write(ToServer(ClientMessage::DevSnowLine(snow_line)));
                }
            },
        );
    }
}
