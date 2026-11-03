//! How things look on starting: herbs, the window, the player's token and the camera.

use bevy::prelude::*;
use bevy::window::PrimaryWindow;
use std::env;

use super::{parse_pos, parse_token};
use crate::common::{GameView, Token};
use crate::player::Player;
use crate::view3d::OrbitCamera;

/// `HOW_DEBUG_HERBS`, `HOW_DEBUG_WINDOW_SIZE`, `HOW_DEBUG_TOKEN` and `HOW_DEBUG_VIEW`.
pub(super) fn add(app: &mut App) {
    if let Some(stages) = env::var("HOW_DEBUG_HERBS")
        .ok()
        .and_then(|n| n.trim().parse::<u32>().ok())
    {
        app.insert_resource(crate::common::HerbGrowthShift(stages));
    }

    if let Some((width, height)) = env::var("HOW_DEBUG_WINDOW_SIZE")
        .ok()
        .and_then(|size| parse_pos(&size))
    {
        let (width, height) = (width as f32, height as f32);
        app.add_systems(
            Update,
            move |time: Res<Time>,
                  mut done: Local<bool>,
                  mut window: Single<&mut Window, With<PrimaryWindow>>| {
                if !*done && time.elapsed_secs() > 1.0 {
                    *done = true;
                    window.set_maximized(false);
                    window.resolution.set(width, height);
                }
            },
        );
    }

    if let Some(new_token) = env::var("HOW_DEBUG_TOKEN")
        .ok()
        .and_then(|t| parse_token(&t))
    {
        app.add_systems(
            OnExit(GameView::Connecting),
            move |mut token: Single<&mut Token, With<Player>>| {
                **token = new_token;
            },
        );
    }

    if let Ok(view) = env::var("HOW_DEBUG_VIEW") {
        let numbers: Vec<f32> = view
            .split(',')
            .filter_map(|n| n.trim().parse().ok())
            .collect();
        app.add_systems(
            OnExit(GameView::Connecting),
            move |mut cameras: Query<&mut OrbitCamera>| {
                for mut camera in &mut cameras {
                    turn_camera(
                        &mut camera,
                        numbers.first().copied(),
                        numbers.get(1).copied(),
                        numbers.get(2).copied(),
                    );
                }
            },
        );
    }
}

/// Turns the camera to `yaw` and `pitch` (degrees) and divides its distance
/// by `zoom`, each if given. Sets the fields directly, so the limits on
/// turning and zooming don't apply.
pub(super) fn turn_camera(
    camera: &mut OrbitCamera,
    yaw: Option<f32>,
    pitch: Option<f32>,
    zoom: Option<f32>,
) {
    if let Some(yaw) = yaw {
        camera.yaw = yaw.to_radians();
    }
    if let Some(pitch) = pitch {
        camera.pitch = pitch.to_radians();
    }
    if let Some(zoom) = zoom {
        camera.distance /= zoom;
    }
}
