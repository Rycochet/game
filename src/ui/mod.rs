//! On-screen interface (HUD, menus). See `docs/context/ui.md`.

use bevy::prelude::*;

pub struct UiPlugin;

impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, spawn_controls_hint);
    }
}

fn spawn_controls_hint(mut commands: Commands) {
    commands.spawn((
        Text::new("Arrow keys / WASD to move"),
        Node {
            position_type: PositionType::Absolute,
            top: px(12),
            left: px(12),
            ..default()
        },
    ));
}
