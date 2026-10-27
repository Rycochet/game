//! Entry point for both the native app and the browser (WebAssembly) build.
//!
//! This file only creates the window and registers each feature module's
//! plugin. Game logic lives in the modules themselves.

mod common;
mod player;
mod ui;

use bevy::prelude::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Game".into(),
                // In the browser, draw into the <canvas id="game"> in index.html.
                canvas: Some("#game".into()),
                fit_canvas_to_parent: true,
                ..default()
            }),
            ..default()
        }))
        .add_plugins((common::CommonPlugin, player::PlayerPlugin, ui::UiPlugin))
        .run();
}
