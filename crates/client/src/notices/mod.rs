//! Short notices at the top middle of the window that go by themselves (a
//! snackbar): for now "Found" when the player finds a poi in the wilderness.
//! See `docs/context/client/notices.md`.

use bevy::prelude::*;
use shared::protocol::ServerMessage;

use crate::common::words;
use crate::connection::FromServer;
use crate::widgets::{INK, PANEL_EDGE, TextSize, panel};

/// How long a notice shows, in seconds.
const SHOWN_FOR: f32 = 4.0;

/// How far below the top of the window it sits, in interface pixels: under
/// the status text.
const FROM_TOP: f32 = 64.0;

/// Notices.
pub struct NoticesPlugin;

impl Plugin for NoticesPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<Shown>()
            .add_systems(Startup, spawn_notice)
            .add_systems(Update, show_notices);
    }
}

/// The row holding the notice, across the window.
#[derive(Component)]
struct NoticeRow;

#[derive(Component)]
struct NoticeText;

/// How long the notice showing has left, in seconds (0: none).
#[derive(Resource, Default)]
struct Shown(f32);

fn spawn_notice(mut commands: Commands) {
    commands
        .spawn((
            NoticeRow,
            Node {
                position_type: PositionType::Absolute,
                top: px(FROM_TOP),
                left: px(0),
                right: px(0),
                justify_content: JustifyContent::Center,
                ..default()
            },
            Visibility::Hidden,
            Pickable::IGNORE,
        ))
        .with_children(|row| {
            // The panel's own `Node` replaced: in the row's flow, not placed.
            row.spawn(panel())
                .insert((
                    Node {
                        border: UiRect::all(px(PANEL_EDGE)),
                        padding: UiRect::axes(px(16), px(8)),
                        ..default()
                    },
                    Pickable::IGNORE,
                ))
                .with_child((
                    NoticeText,
                    Text::new(""),
                    TextSize::Medium.font(),
                    TextColor(INK),
                ));
        });
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
    mut row: Single<&mut Visibility, With<NoticeRow>>,
    mut text: Single<&mut Text, With<NoticeText>>,
) {
    for FromServer(message) in from_server.read() {
        match message {
            &ServerMessage::PoiFound { kind } => text.0 = found(kind),
            ServerMessage::DoorRefused { locked: true, .. } => text.0 = LOCKED.to_owned(),
            _ => continue,
        }
        shown.0 = SHOWN_FOR;
    }
    if shown.0 > 0.0 {
        shown.0 -= time.delta_secs();
        row.set_if_neq(Visibility::Inherited);
    } else {
        row.set_if_neq(Visibility::Hidden);
    }
}

#[cfg(test)]
mod tests;
