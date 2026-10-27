//! Shared building blocks that every other module may use.
//!
//! Keep this small: only types that genuinely need to be shared between
//! feature modules belong here. See `docs/context/common.md`.

use bevy::prelude::*;

pub struct CommonPlugin;

impl Plugin for CommonPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, spawn_camera);
    }
}

/// The single 2D camera that renders the game.
fn spawn_camera(mut commands: Commands) {
    commands.spawn(Camera2d);
}
