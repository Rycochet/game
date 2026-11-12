//! Short notices at the top middle of the window that go by themselves (a
//! snackbar): for now "Found" when the player finds a poi in the wilderness.
//! See `docs/context/client/notices.md`.

use bevy::prelude::*;
use shared::protocol::ServerMessage;

use crate::common::words;
use crate::connection::FromServer;
use crate::dom::{Dom, Template, document};
use crate::html;

/// How long a notice shows, in seconds.
const SHOWN_FOR: f32 = 4.0;

/// Notices.
pub struct NoticesPlugin;

impl Plugin for NoticesPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<Shown>()
            .add_systems(Startup, spawn_notice)
            .add_systems(Update, show_notices);
    }
}

/// The notice: a panel in the middle of a row across the window, under the
/// status text. The pointer is never on it (`inert`).
static NOTICES: Template = html!(
    "notices",
    r#"
<form id="notices">
  <div id="row" top="64" left="0" right="0" align="center" inert hidden>
    <panel id="notice"><p id="words" class="medium"/></panel>
  </div>
</form>
"#
);

/// How long the notice showing has left, in seconds (0: none).
#[derive(Resource, Default)]
struct Shown(f32);

fn spawn_notice(mut commands: Commands) {
    // Over the whole window, for the row to be placed in.
    let fills = Node {
        position_type: PositionType::Absolute,
        width: percent(100),
        height: percent(100),
        // (Stacked, so the form in it is as wide as the window.)
        flex_direction: FlexDirection::Column,
        ..default()
    };
    commands.spawn(document(&NOTICES)).insert(fills);
}

/// What a found poi's notice says.
fn found(kind: shared::poi::PoiKind) -> String {
    format!("Found: {}", words(&format!("{kind:?}")))
}

/// What a door found locked says.
const LOCKED: &str = "Locked";

/// Shows a notice when the server says something worth one, the newest in
/// place of any showing, and hides it after `SHOWN_FOR`.
fn show_notices(
    mut from_server: MessageReader<FromServer>,
    time: Res<Time>,
    mut shown: ResMut<Shown>,
    mut dom: Dom,
    mut showing: Local<bool>,
) {
    for FromServer(message) in from_server.read() {
        let words = match message {
            &ServerMessage::PoiFound { kind } => found(kind),
            ServerMessage::DoorRefused { locked: true, .. } => LOCKED.to_owned(),
            _ => continue,
        };
        dom.set_text("notices/words", words);
        shown.0 = SHOWN_FOR;
    }
    if shown.0 > 0.0 {
        shown.0 -= time.delta_secs();
    }
    let now = shown.0 > 0.0;
    if *showing != now {
        *showing = now;
        dom.set_flag("notices/row", "hidden", !now);
    }
}

#[cfg(test)]
mod tests;
