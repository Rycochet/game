//! The account in the bottom left corner: "Logged in as" its email address, at
//! the startup menu and while making a character. Never in game. The menu's
//! Logout forgets it (`system_menu`).

use bevy::prelude::*;

use crate::common::{AccountEmail, MakingCharacter, Session};
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

/// Shows the account at the startup menu and while making a character.
pub(super) fn show_corner(
    email: Res<AccountEmail>,
    session: Res<Session>,
    making: Res<MakingCharacter>,
    mut words: Single<(&mut Text, &mut Visibility), With<CornerWords>>,
) {
    if !email.is_changed() && !session.is_changed() && !making.is_changed() {
        return;
    }
    let shown = email.0.is_some() && (!session.is_playing() || making.0);
    words.1.set_if_neq(if shown {
        Visibility::Inherited
    } else {
        Visibility::Hidden
    });
    if let Some(email) = &email.0 {
        words.0.0 = format!("Logged in as {email}");
    }
}
