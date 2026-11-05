//! "Online" (native only): a toggle with Server Options' gear attached, opening
//! this game's own server to other players. The ports open once playing on it
//! (`connection` asks); while playing there, turning it either way asks first.
//! If it can't open, it goes back off and says why.

use bevy::prelude::*;
use shared::protocol::OnlineState;

use super::{
    ABOVE_MENU_LAYER, Action, BUTTON_ROW, FONT, Layer, MENU_WIDTH, MenuButton, SystemMenu,
};
use crate::common::{Hosting, Preferences, ServerClock, Session};
use crate::widgets::{
    ButtonPressed, ButtonState, ButtonStyle, ButtonTooltip, INK, Pressable, spawn_button,
    spawn_dialog, spawn_icon_button,
};

const WORDS_WIDTH: f32 = 420.0;

/// The toggle itself.
#[derive(Component)]
pub(super) struct OnlineToggle;

/// Why it couldn't open.
#[derive(Component)]
pub(super) struct FailedWords;

/// The toggle and the gear, as wide together as the menu's other buttons.
pub(super) fn spawn_online_row(section: &mut ChildSpawnerCommands, gear: Handle<Image>) {
    section
        .spawn(Node {
            width: percent(100),
            height: px(BUTTON_ROW),
            ..default()
        })
        .with_children(|row| {
            let node = Node {
                flex_grow: 1.0,
                height: px(BUTTON_ROW),
                ..default()
            };
            spawn_button(row, ButtonState::Normal, "Online", FONT, node).insert((
                OnlineToggle,
                MenuButton::Online,
                Action::ToggleOnline,
                Action::ToggleOnline.ui_id("menu"),
                Pressable,
                ButtonTooltip("Let other players join this game".into()),
            ));
            let node = Node {
                width: px(BUTTON_ROW),
                height: px(BUTTON_ROW),
                flex_shrink: 0.0,
                ..default()
            };
            spawn_icon_button(row, ButtonState::Normal, gear, node).insert((
                Action::ServerOptions,
                Action::ServerOptions.ui_id("menu"),
                Pressable,
                ButtonTooltip("Server Options".into()),
            ));
        });
}

pub(super) fn spawn_online_requesters(commands: &mut Commands) {
    let asks = [
        (
            SystemMenu::GoingOnline,
            "going_online",
            "Go online",
            "This will let other players connect to this game. Are you sure?",
        ),
        (
            SystemMenu::GoingOffline,
            "going_offline",
            "Go offline",
            "Anyone else playing will be told the server is going offline, and disconnected. \
             Are you sure?",
        ),
    ];
    for (state, form, title, words) in asks {
        let dialog = spawn_dialog(commands, ABOVE_MENU_LAYER, title);
        commands
            .entity(dialog.layer)
            .insert((Layer(state_slice(state)), Action::Back));
        commands.entity(dialog.content).with_child((
            Text::new(words),
            FONT.font(),
            TextColor(INK),
            Node {
                width: px(WORDS_WIDTH),
                ..default()
            },
        ));
        commands.entity(dialog.buttons).with_children(|row| {
            for (label, action, style) in [
                ("Yes", Action::ReallyToggleOnline, ButtonStyle::Main),
                ("No", Action::Back, ButtonStyle::Plain),
            ] {
                let node = Node {
                    width: px(MENU_WIDTH / 2.0),
                    height: px(BUTTON_ROW),
                    ..default()
                };
                spawn_button(row, ButtonState::Normal, label, FONT, node).insert((
                    action,
                    style,
                    Pressable,
                    action.ui_id(form),
                ));
            }
        });
    }
    let dialog = spawn_dialog(commands, ABOVE_MENU_LAYER, "Can't go online");
    commands
        .entity(dialog.layer)
        .insert((Layer(&[SystemMenu::OnlineFailed]), Action::Back));
    commands.entity(dialog.content).with_child((
        FailedWords,
        Text::new(""),
        FONT.font(),
        TextColor(INK),
        Node {
            width: px(WORDS_WIDTH),
            ..default()
        },
    ));
    commands.entity(dialog.buttons).with_children(|row| {
        let node = Node {
            width: px(MENU_WIDTH / 2.0),
            height: px(BUTTON_ROW),
            ..default()
        };
        spawn_button(row, ButtonState::Normal, "OK", FONT, node).insert((
            Action::Back,
            Action::Back.ui_id("online_failed"),
            ButtonStyle::Main,
            Pressable,
        ));
    });
}

/// `Layer` wants a list that lasts.
fn state_slice(state: SystemMenu) -> &'static [SystemMenu] {
    match state {
        SystemMenu::GoingOnline => &[SystemMenu::GoingOnline],
        _ => &[SystemMenu::GoingOffline],
    }
}

/// The toggle's presses: at once unless playing on this game's own server, where
/// it asks first.
pub(super) fn use_online(
    mut pressed: MessageReader<ButtonPressed>,
    actions: Query<&Action>,
    mut menu: ResMut<SystemMenu>,
    session: Res<Session>,
    clock: Res<ServerClock>,
    mut preferences: ResMut<Preferences>,
) {
    let action = pressed
        .read()
        .filter_map(|press| actions.get(press.0).ok())
        .last()
        .copied();
    match (action, *menu) {
        (Some(Action::ToggleOnline), SystemMenu::Menu) => {
            if session.is_playing() && clock.local_server {
                *menu = if preferences.online {
                    SystemMenu::GoingOffline
                } else {
                    SystemMenu::GoingOnline
                };
            } else {
                preferences.online = !preferences.online;
            }
        }
        (Some(Action::ReallyToggleOnline), SystemMenu::GoingOnline | SystemMenu::GoingOffline) => {
            preferences.online = *menu == SystemMenu::GoingOnline;
            *menu = SystemMenu::Menu;
        }
        _ => {}
    }
}

/// The toggle is held down while on.
pub(super) fn show_online(
    preferences: Res<Preferences>,
    mut toggles: Query<&mut ButtonState, With<OnlineToggle>>,
) {
    for mut state in &mut toggles {
        state.set_if_neq(if preferences.online {
            ButtonState::Selected
        } else {
            ButtonState::Normal
        });
    }
}

/// If the server couldn't open (its port is taken, say), Online goes back off
/// and the menu says why.
pub(super) fn show_online_failure(
    hosting: Res<Hosting>,
    mut preferences: ResMut<Preferences>,
    mut menu: ResMut<SystemMenu>,
    mut words: Single<&mut Text, With<FailedWords>>,
) {
    if !hosting.is_changed() {
        return;
    }
    if let OnlineState::Failed(reason) = &hosting.0 {
        words.0 = format!("{reason}. Online is off again: change Server Options and try again.");
        preferences.online = false;
        *menu = SystemMenu::OnlineFailed;
    }
}
