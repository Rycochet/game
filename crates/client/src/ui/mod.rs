//! On-screen interface (HUD, menus). See `docs/context/client/ui.md`.

mod frame_rate;

use bevy::prelude::*;
use frame_rate::FrameTimes;
use shared::role::DebugTool;

use crate::common::{
    ClickMode, DebugInfo, DebugTools, FreezeMobs, GameClock, GameView, GodMode, MapPosition,
    ServerClock, Truesight, ViewStats,
};
use crate::dom::{Dom, Template, document};
use crate::html;
use crate::player::Player;

pub struct UiPlugin;

impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<FrameTimes>()
            .add_systems(Startup, spawn_status_text)
            .add_systems(Update, (frame_rate::note_frame, update_status_text).chain());
    }
}

/// The status text: its first line, with what it warns of after it in red,
/// and under them the view's numbers. White over the game's view, never taking
/// the pointer (`inert`).
static STATUS: Template = html!(
    "hud.status",
    r#"
<form id="status">
  <div id="text" class="joined" top="16" left="56" inert>
    <span class="joined">
      <p id="words" class="status"/>
      <p id="warning" class="status-warning"/>
    </span>
    <p id="stats" class="status"/>
  </div>
</form>
"#
);

/// Where the status text goes, to clear the warning button (as the markup
/// starts it).
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

fn spawn_status_text(mut commands: Commands) {
    // Over the whole window, stacked, for the text to be placed in.
    let fills = Node {
        position_type: PositionType::Absolute,
        width: percent(100),
        height: percent(100),
        flex_direction: FlexDirection::Column,
        ..default()
    };
    commands.spawn(document(&STATUS)).insert(fills);
}

/// What the status text last said, and where it was.
#[derive(Default, PartialEq)]
struct Said {
    words: String,
    warning: String,
    stats: String,
    left: f32,
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
    mut dom: Dom,
    mut said: Local<Option<Said>>,
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
    let left = if tools.any() {
        STATUS_LEFT
    } else {
        STATUS_LEFT_ALONE
    };
    // Nothing else to say before the first welcome.
    let connecting = *view.get() == GameView::Connecting;
    let warning = match (connecting, mode.is_empty()) {
        (true, _) | (_, true) => mode,
        (false, false) => format!("  {mode}"),
    };
    let (words, stats) = if connecting {
        (String::new(), String::new())
    } else {
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
        // Show Debug Info adds the hour and the view's numbers, for those who
        // may use it.
        let info = info.0 && tools.may(DebugTool::ShowDebugInfo);
        if info {
            status += &format!("  {HOUR_ICON} {}d{:02}h", clock.hour / 24, clock.hour % 24);
        }
        let stats = match info {
            true => stats.summary.clone(),
            false => String::new(),
        };
        (status, stats)
    };
    let now = Said {
        words,
        warning,
        stats,
        left,
    };
    // Only what differs from what it says is changed: most frames, nothing.
    let was = said.as_ref();
    if was.is_none_or(|was| was.left != now.left) {
        dom.set("status/text", "left", &now.left.to_string());
    }
    if was.is_none_or(|was| was.words != now.words) {
        dom.set_text("status/words", now.words.clone());
    }
    if was.is_none_or(|was| was.warning != now.warning) {
        dom.set_text("status/warning", now.warning.clone());
    }
    if was.is_none_or(|was| was.stats != now.stats) {
        dom.set_text("status/stats", now.stats.clone());
    }
    *said = Some(now);
}

#[cfg(test)]
mod tests;
