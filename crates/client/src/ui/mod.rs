//! On-screen interface (HUD, menus). See `docs/context/client/ui.md`.

use bevy::prelude::*;

use crate::common::{
    ClickMode, DebugInfo, GameClock, GameView, MapPosition, ServerClock, ViewStats, Weather,
};
use crate::debug_menu::TemperatureCycle;
use crate::fonts;
use crate::player::Player;

pub struct UiPlugin;

impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, spawn_status_text)
            .add_systems(Update, update_status_text);
    }
}

#[derive(Component)]
struct StatusText;

/// After the status text's first line, in red: the mode clicks are in, if any.
#[derive(Component)]
struct ModeText;

/// The status text's second line.
#[derive(Component)]
struct StatsText;

/// Where the status text goes, to clear the warning button.
const STATUS_LEFT: f32 = 56.0;

/// What the status text's numbers are, as symbols.
const POSITION_ICON: char = '\u{1F4CD}';
const HOUR_ICON: char = '\u{1F550}';
const PING_ICON: char = '\u{1F4F6}';
const SNOW_LINE_ICON: char = '\u{2744}';

const MODE_COLOUR: Color = Color::srgb(1.0, 0.2, 0.2);

fn spawn_status_text(mut commands: Commands) {
    commands.spawn((
        StatusText,
        Text::new(""),
        TextFont {
            font: fonts::with_icons(),
            ..default()
        },
        Node {
            position_type: PositionType::Absolute,
            top: px(16),
            left: px(STATUS_LEFT),
            ..default()
        },
        children![
            (ModeText, TextSpan::default(), TextColor(MODE_COLOUR)),
            (StatsText, TextSpan::default()),
        ],
    ));
}

#[allow(clippy::too_many_arguments)]
fn update_status_text(
    player: Single<&MapPosition, With<Player>>,
    clock: Res<GameClock>,
    server_clock: Res<ServerClock>,
    weather: Res<Weather>,
    cycle: Res<TemperatureCycle>,
    stats: Res<ViewStats>,
    info: Res<DebugInfo>,
    view: Res<State<GameView>>,
    mode: Res<ClickMode>,
    mut text: Single<&mut Text, With<StatusText>>,
    mut mode_text: Single<&mut TextSpan, (With<ModeText>, Without<StatsText>)>,
    mut stats_text: Single<&mut TextSpan, (With<StatsText>, Without<ModeText>)>,
) {
    // Says when clicks don't do what they usually do, as it is easy to forget:
    // even with the rest hidden.
    let mode = match *mode {
        ClickMode::Normal => "",
        ClickMode::Teleport => "Teleport Mode",
        ClickMode::ChangeCell => "Change Cell Mode",
    };
    // Nothing else to say before the first welcome.
    if !info.0 || *view.get() == GameView::Connecting {
        text.0 = String::new();
        stats_text.0 = String::new();
        mode_text.0 = mode.into();
        return;
    }
    mode_text.0 = if mode.is_empty() {
        String::new()
    } else {
        format!("  {mode}")
    };
    // Single player has no network to time.
    let ping = match server_clock.round_trip_ms {
        Some(ms) if !server_clock.local_server => format!("{ms:.0}ms"),
        _ => "-".to_string(),
    };
    let mut status = format!(
        "{POSITION_ICON} {},{}  {HOUR_ICON} {}  {PING_ICON} {ping}",
        player.x,
        player.y,
        clock.hour % 24
    );
    if *cycle != TemperatureCycle::Off {
        status += &format!("  {SNOW_LINE_ICON} {:.0}", weather.snow_line);
    }
    text.0 = status;
    stats_text.0 = format!("\n{}", stats.summary);
}
