//! On-screen interface (HUD, menus). See `docs/context/client/ui.md`.

mod frame_rate;

use bevy::prelude::*;
use frame_rate::FrameTimes;
use shared::role::DebugTool;

use crate::common::{
    ClickMode, DebugInfo, DebugTools, FreezeMobs, GameClock, GameView, GodMode, MapPosition,
    ServerClock, Truesight, ViewStats,
};
use crate::fonts;
use crate::player::Player;

pub struct UiPlugin;

impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<FrameTimes>()
            .add_systems(Startup, spawn_status_text)
            .add_systems(Update, (frame_rate::note_frame, update_status_text).chain());
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
/// Where it goes with no warning button: no debug tools, so no debug menu.
const STATUS_LEFT_ALONE: f32 = 16.0;

/// What the status text's numbers are, as symbols.
const POSITION_ICON: char = '\u{1F4CD}';
const HOUR_ICON: char = '\u{1F550}';
const PING_ICON: char = '\u{1F4F6}';
/// A clapperboard for frames: the icon font has no stopwatch (owner's choice).
const FRAME_RATE_ICON: char = '\u{1F3AC}';
const GOD_MODE_ICON: &str = "\u{1F47B}";
const TRUESIGHT_ICON: &str = "\u{1F440}";

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
            (
                ModeText,
                TextSpan::default(),
                TextColor(MODE_COLOUR),
                // Each span has its own font, and this one shows the ghost and skull.
                TextFont {
                    font: fonts::with_icons(),
                    ..default()
                },
            ),
            (StatsText, TextSpan::default()),
        ],
    ));
}

#[allow(clippy::too_many_arguments)]
fn update_status_text(
    player: Single<&MapPosition, With<Player>>,
    clock: Res<GameClock>,
    server_clock: Res<ServerClock>,
    stats: Res<ViewStats>,
    (info, frames, tools): (Res<DebugInfo>, Res<FrameTimes>, Res<DebugTools>),
    view: Res<State<GameView>>,
    mode: Res<ClickMode>,
    god: Res<GodMode>,
    (truesight, freeze): (Res<Truesight>, Res<FreezeMobs>),
    mut text: Single<(&mut Text, &mut Node), With<StatusText>>,
    mut mode_text: Single<&mut TextSpan, (With<ModeText>, Without<StatsText>)>,
    mut stats_text: Single<&mut TextSpan, (With<StatsText>, Without<ModeText>)>,
) {
    // Says when clicks don't do what they usually do, as it is easy to forget:
    // even before the first welcome.
    let mode = match *mode {
        ClickMode::Normal => "",
        ClickMode::Teleport => "Teleport Mode",
        ClickMode::ChangeCell => "Change Cell Mode",
        ClickMode::PlaceMob => "Mob Mode",
    };
    let says = [
        (god.0, GOD_MODE_ICON),
        (truesight.0, TRUESIGHT_ICON),
        (freeze.0, "Mobs Frozen"),
        (!mode.is_empty(), mode),
    ];
    let says: Vec<&str> = says
        .into_iter()
        .filter_map(|(on, words)| on.then_some(words))
        .collect();
    let mode = says.join(" ");
    let (text, node) = &mut *text;
    let left = if tools.any() {
        STATUS_LEFT
    } else {
        STATUS_LEFT_ALONE
    };
    if node.left != px(left) {
        node.left = px(left);
    }
    // Nothing else to say before the first welcome.
    if *view.get() == GameView::Connecting {
        text.0 = String::new();
        stats_text.0 = String::new();
        mode_text.0 = mode;
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
    let rate = frames
        .rate()
        .map_or("-".to_string(), |rate| format!("{rate:.0}"));
    let mut status = format!(
        "{POSITION_ICON} {},{}  {FRAME_RATE_ICON} {rate}  {PING_ICON} {ping}",
        player.x, player.y,
    );
    // Show Debug Info adds the hour and the view's numbers, for those who may
    // use it.
    let info = info.0 && tools.may(DebugTool::ShowDebugInfo);
    if info {
        status += &format!("  {HOUR_ICON} {}d{:02}h", clock.hour / 24, clock.hour % 24);
    }
    text.0 = status;
    stats_text.0 = if info {
        format!("\n{}", stats.summary)
    } else {
        String::new()
    };
}
