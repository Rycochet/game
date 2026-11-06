//! Delete Character: asks first, then has the server delete the character and
//! ask for a new one, which `character_create` makes. Until there is a screen for
//! choosing characters, this is how a new one is made by hand.

use bevy::prelude::*;
use shared::protocol::ClientMessage;

use super::DebugAction;
use crate::connection::ToServer;
use crate::widgets::{
    ButtonPressed, ButtonState, ButtonStyle, INK, Pressable, TextSize, UiId, spawn_button,
    spawn_dialog,
};

/// Over the debug menu's own layer.
const LAYER: i32 = 1;
const FORM: &str = "debug.delete_character";
const BUTTON_WIDTH: f32 = 110.0;

#[derive(Component)]
pub(super) struct DeleteDialog;
#[derive(Component)]
pub(super) struct ReallyDelete;
#[derive(Component)]
pub(super) struct KeepIt;

pub(super) fn spawn_delete_dialog(mut commands: Commands) {
    let dialog = spawn_dialog(&mut commands, LAYER, "Delete Character");
    commands.entity(dialog.layer).insert(DeleteDialog);
    commands.entity(dialog.content).with_child((
        Text::new("Delete your character? This can't be undone.\nYou then make a new one."),
        TextSize::Medium.font(),
        TextColor(INK),
    ));
    let node = || Node {
        width: px(BUTTON_WIDTH),
        padding: UiRect::vertical(px(4)),
        ..default()
    };
    commands.entity(dialog.buttons).with_children(|row| {
        spawn_button(row, ButtonState::Normal, "Delete", TextSize::Medium, node()).insert((
            ReallyDelete,
            ButtonStyle::Main,
            Pressable,
            UiId::new(FORM, "delete"),
        ));
        spawn_button(row, ButtonState::Normal, "Cancel", TextSize::Medium, node()).insert((
            KeepIt,
            Pressable,
            UiId::new(FORM, "cancel"),
        ));
    });
}

/// Opens on the menu's entry; Delete sends it, Cancel or a press outside just
/// closes it.
pub(super) fn run_delete_dialog(
    mut actions: MessageReader<DebugAction>,
    mut pressed: MessageReader<ButtonPressed>,
    mut dialog: Single<(Entity, &mut Visibility), With<DeleteDialog>>,
    delete: Single<Entity, With<ReallyDelete>>,
    keep: Single<Entity, With<KeepIt>>,
    mut to_server: MessageWriter<ToServer>,
) {
    if actions
        .read()
        .any(|&action| action == DebugAction::DeleteCharacter)
    {
        *dialog.1 = Visibility::Inherited;
    }
    for press in pressed.read() {
        if press.0 == *delete {
            to_server.write(ToServer(ClientMessage::DevDeleteCharacter));
        } else if press.0 != *keep && press.0 != dialog.0 {
            continue;
        }
        *dialog.1 = Visibility::Hidden;
    }
}
