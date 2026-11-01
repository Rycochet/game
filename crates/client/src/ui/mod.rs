//! On-screen interface (HUD, menus). See `docs/context/client/ui.md`.

use bevy::prelude::*;

use crate::common::{
    GameClock, GameView, MapPosition, ServerClock, ServerStatus, ViewStats, Weather,
};
use crate::demo::{AutoWalk, TemperatureCycle};
use crate::player::Player;

pub struct UiPlugin;

impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, (spawn_status_text, spawn_connection_text))
            .add_systems(Update, (update_status_text, update_connection_text));
    }
}

#[derive(Component)]
struct StatusText;

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

fn spawn_status_text(mut commands: Commands) {
    commands.spawn((
        StatusText,
        Text::new(""),
        Node {
            position_type: PositionType::Absolute,
            top: px(12),
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
    walk: Res<AutoWalk>,
    cycle: Res<TemperatureCycle>,
    stats: Res<ViewStats>,
    view: Res<State<GameView>>,
    mut text: Single<&mut Text, With<StatusText>>,
) {
    // Nothing to say before the first welcome (`ConnectionText` says why).
    if *view.get() == GameView::Connecting {
        text.0 = String::new();
        return;
    }
    let on_off = |on: bool| if on { "on" } else { "off" };
    let cycle = match *cycle {
        TemperatureCycle::Off => "off",
        TemperatureCycle::Cooling => "cooling",
        TemperatureCycle::Warming => "warming",
    };
    text.0 = format!(
        "Arrow keys / WASD or click to move  Click a place or stairs to go in\n\
         Right-drag: turn / tilt map  Wheel: zoom  Home: reset view\n\
         Ctrl+click: teleport  Alt+click: change a site's cell  Esc: back to the start\n\
         G: walk ({})  T: temperature cycle ({cycle}, snow line {:.0})  P: poi showcase\n\
         M: map  K / C / X / Z: token shape / colour / completed / size  B: stairs style\n\
         Position {}, {}  Hour {}  Ping {}\n{}",
        on_off(walk.0),
        weather.snow_line,
        player.x,
        player.y,
        clock.hour % 24,
        server_clock
            .round_trip_ms
            .map_or("-".to_string(), |ms| format!("{ms:.0} ms")),
        stats.summary
    );
}
