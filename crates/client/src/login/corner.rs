//! The bottom left corner, at the startup menu and while making a character:
//! "Playing on" the server Play goes to, if it isn't the usual one, and
//! "Logged in as" the account's email address. Never in game. The menu's Logout
//! forgets both (`system_menu`, `connection`).

use bevy::prelude::*;

use crate::common::{AccountEmail, ChosenServer, MakingCharacter, Session};
use crate::widgets::{TextSize, layer_z};

const MARGIN: f32 = 16.0;

/// Light, to read on the title's dark background.
const WORDS: Color = Color::srgb(0.93, 0.9, 0.82);

#[derive(Component)]
pub(super) struct CornerWords;

pub(super) fn spawn_corner(mut commands: Commands) {
    commands.spawn((
        CornerWords,
        Text::new(""),
        TextSize::Medium.font(),
        TextColor(WORDS),
        Node {
            position_type: PositionType::Absolute,
            left: px(MARGIN),
            bottom: px(MARGIN),
            ..default()
        },
        // Over the startup menu's layer, under the requesters above it.
        GlobalZIndex(layer_z(1) - 1),
        Visibility::Hidden,
    ));
}

/// Shows the server and account at the startup menu and while making a
/// character (only there: CLAUDE.md's rule on email addresses).
pub(super) fn show_corner(
    email: Res<AccountEmail>,
    chosen: Res<ChosenServer>,
    session: Res<Session>,
    making: Res<MakingCharacter>,
    mut words: Single<(&mut Text, &mut Visibility), With<CornerWords>>,
) {
    if !email.is_changed() && !chosen.is_changed() && !session.is_changed() && !making.is_changed()
    {
        return;
    }
    let mut lines = Vec::new();
    if let Some(server) = &chosen.0 {
        lines.push(format!("Playing on {}", server.label()));
    }
    if let Some(email) = &email.0 {
        lines.push(format!("Logged in as {email}"));
    }
    let shown = !lines.is_empty() && (!session.is_playing() || making.0);
    words.1.set_if_neq(if shown {
        Visibility::Inherited
    } else {
        Visibility::Hidden
    });
    words.0.0 = lines.join("\n");
}
