//! When a server won't be played on (it refuses, or its certificate isn't the
//! one kept for it): back to the startup menu, with a requester saying why.

use bevy::prelude::*;

use super::{ABOVE_MENU_LAYER, Action, BUTTON_ROW, FONT, Layer, SystemMenu};
use crate::common::{ChosenServer, SavedServer, ServerStatus, Session};
use crate::widgets::{ButtonState, ButtonStyle, Pressable, spawn_button, spawn_dialog};

const WORDS_WIDTH: f32 = 460.0;
/// Dark red, to read on the parchment.
const WORDS_COLOUR: Color = Color::srgb(0.62, 0.1, 0.06);

/// Why.
#[derive(Component)]
pub(super) struct RefusalWords;

/// The server chosen in Connect that just refused, if it was one: OK goes back
/// to Connect with it in the box (and its key in the padlock), as going back to
/// the startup menu forgets `ChosenServer`.
#[derive(Resource, Default)]
pub(super) struct TriedServer(pub(super) Option<SavedServer>);

pub(super) fn spawn_refusal(commands: &mut Commands) {
    let dialog = spawn_dialog(commands, ABOVE_MENU_LAYER, "Can't connect");
    commands
        .entity(dialog.layer)
        .insert((Layer(&[SystemMenu::Refused]), Action::Back));
    commands.entity(dialog.content).with_child((
        RefusalWords,
        Text::new(""),
        FONT.font(),
        TextColor(WORDS_COLOUR),
        Node {
            width: px(WORDS_WIDTH),
            ..default()
        },
    ));
    commands.entity(dialog.buttons).with_children(|row| {
        let node = Node {
            width: px(WORDS_WIDTH / 4.0),
            height: px(BUTTON_ROW),
            ..default()
        };
        spawn_button(row, ButtonState::Normal, "OK", FONT, node).insert((
            Action::Back,
            Action::Back.ui_id("refused"),
            ButtonStyle::Main,
            Pressable,
        ));
    });
}

/// A refusal while playing (or trying to) goes back to the startup menu to say
/// why. `connection` keeps the refusal until playing again.
pub(super) fn show_refusal(
    status: Res<ServerStatus>,
    chosen: Res<ChosenServer>,
    mut tried: ResMut<TriedServer>,
    mut session: ResMut<Session>,
    mut menu: ResMut<SystemMenu>,
    mut words: Single<&mut Text, With<RefusalWords>>,
) {
    if !status.is_changed() || !session.is_playing() {
        return;
    }
    if let ServerStatus::Refused(reason) = &*status {
        words.0 = reason.clone();
        tried.0.clone_from(&chosen.0);
        *session = Session::AtStart;
        *menu = SystemMenu::Refused;
    }
}
