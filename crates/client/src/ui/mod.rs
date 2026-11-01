//! On-screen interface (HUD, menus). See `docs/context/client/ui.md`.

use bevy::prelude::*;

use crate::common::{
    ClickMode, DebugInfo, GameClock, GameView, MapPosition, ServerClock, ServerStatus, ViewStats,
    Weather,
};
use crate::player::Player;

pub struct UiPlugin;

impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Startup,
            (spawn_status_text, spawn_mode_text, spawn_connection_text),
        )
        .add_systems(
            Update,
            (update_status_text, update_mode_text, update_connection_text),
        );
    }
}

#[derive(Component)]
struct StatusText;

/// Beside the warning button, in red: the mode clicks are in, if any.
#[derive(Component)]
struct ModeText;

/// Where the status text and mode text go, to clear the warning button.
const STATUS_TOP: f32 = 52.0;
const MODE_LEFT: f32 = 56.0;

/// Bottom left, in red: "Connecting..." while not connected to the server,
/// or why the server refused.
#[derive(Component)]
struct ConnectionText;

const CONNECTION_COLOUR: Color = Color::srgb(1.0, 0.2, 0.2);

fn spawn_connection_text(mut commands: Commands) {
    commands.spawn((
        ConnectionText,
        Text::new(""),
        TextColor(CONNECTION_COLOUR),
        Node {
            position_type: PositionType::Absolute,
            bottom: px(12),
            left: px(12),
            ..default()
        },
    ));
}

fn update_connection_text(
    status: Res<ServerStatus>,
    mut text: Single<&mut Text, With<ConnectionText>>,
) {
    if !status.is_changed() {
        return;
    }
    text.0 = match &*status {
        ServerStatus::Connecting => "Connecting...".into(),
        ServerStatus::Connected => String::new(),
        ServerStatus::Refused(reason) => format!("The server refused: {reason}"),
    };
}

fn spawn_mode_text(mut commands: Commands) {
    commands.spawn((
        ModeText,
        Text::new(""),
        TextColor(CONNECTION_COLOUR),
        Node {
            position_type: PositionType::Absolute,
            top: px(16),
            left: px(MODE_LEFT),
            ..default()
        },
    ));
}

fn spawn_status_text(mut commands: Commands) {
    commands.spawn((
        StatusText,
        Text::new(""),
        Node {
            position_type: PositionType::Absolute,
            top: px(STATUS_TOP),
            left: px(12),
            ..default()
        },
    ));
}

#[allow(clippy::too_many_arguments)]
fn update_status_text(
    player: Single<&MapPosition, With<Player>>,
    clock: Res<GameClock>,
    server_clock: Res<ServerClock>,
    weather: Res<Weather>,
    stats: Res<ViewStats>,
    info: Res<DebugInfo>,
    view: Res<State<GameView>>,
    mut text: Single<&mut Text, With<StatusText>>,
) {
    // Nothing to say before the first welcome (`ConnectionText` says why).
    if !info.0 || *view.get() == GameView::Connecting {
        text.0 = String::new();
        return;
    }
    text.0 = format!(
        "Position {}, {}  Hour {}  Ping {}  Snow line {:.0}\n{}",
        player.x,
        player.y,
        clock.hour % 24,
        server_clock
            .round_trip_ms
            .map_or("-".to_string(), |ms| format!("{ms:.0} ms")),
        weather.snow_line,
        stats.summary
    );
}

/// Says when clicks don't do what they usually do, as it is easy to forget.
fn update_mode_text(mode: Res<ClickMode>, mut text: Single<&mut Text, With<ModeText>>) {
    if !mode.is_changed() {
        return;
    }
    text.0 = match *mode {
        ClickMode::Normal => "",
        ClickMode::Teleport => "Teleport Mode",
        ClickMode::ChangeCell => "Change Cell Mode",
    }
    .into();
}
