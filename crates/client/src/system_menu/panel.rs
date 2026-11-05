//! The menu's own panel: "Paused" above three sections of buttons, with a gap
//! between them.

use bevy::prelude::*;

use super::spawn::{spawn_layer, wide_button};
use super::{
    Action, FONT, GAP, MARGIN, MENU_LAYER, MENU_WIDTH, MenuButton, PAUSED_ROW, PausedText,
    SECTION_GAP, SystemMenu,
};
use crate::widgets::{ButtonState, Pressable, panel_title, spawn_button};

/// One button: what it says, what it does, and when it shows if not always.
type Entry = (&'static str, Action, Option<MenuButton>);

/// The sections, top to bottom.
const SECTIONS: &[&[Entry]] = &[
    &[
        ("Play", Action::Play, Some(MenuButton::Play)),
        ("Continue", Action::Back, Some(MenuButton::Continue)),
        // With the gear for Server Options; native only.
        ("Online", Action::ToggleOnline, Some(MenuButton::Online)),
        ("Connect...", Action::Connect, Some(MenuButton::Connect)),
    ],
    &[
        ("Settings", Action::Settings, None),
        ("Credits", Action::Credits, None),
    ],
    &[
        ("Logout", Action::LogOut, Some(MenuButton::LogOut)),
        ("Quit", Action::Quit, None),
    ],
];

/// `gear` is for Online's Server Options, which a browser hasn't.
#[cfg_attr(target_arch = "wasm32", allow(unused_variables))]
pub(super) fn spawn_main_panel(commands: &mut Commands, gear: Handle<Image>) {
    let panel = spawn_layer(
        commands,
        MENU_LAYER,
        &[
            SystemMenu::Menu,
            SystemMenu::Confirming,
            SystemMenu::Settings,
            SystemMenu::KeepVideo,
            SystemMenu::Credits,
            SystemMenu::Connect,
            SystemMenu::Certificate,
            SystemMenu::Forgetting,
            SystemMenu::Refused,
            SystemMenu::ServerOptions,
            SystemMenu::GoingOnline,
            SystemMenu::GoingOffline,
            SystemMenu::OnlineFailed,
        ],
        px(MENU_WIDTH),
        Val::Auto,
        Node {
            padding: UiRect::all(px(MARGIN)),
            flex_direction: FlexDirection::Column,
            row_gap: px(SECTION_GAP),
            ..default()
        },
    );
    commands.entity(panel).with_children(|panel| {
        panel
            .spawn((
                PausedText,
                Node {
                    // Taking no room until the game is paused (owner's choice).
                    display: Display::None,
                    height: px(PAUSED_ROW),
                    flex_shrink: 0.0,
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    ..default()
                },
            ))
            .with_child(panel_title("Paused"));
        for section in SECTIONS {
            panel
                .spawn(Node {
                    flex_direction: FlexDirection::Column,
                    row_gap: px(GAP),
                    ..default()
                })
                .with_children(|section_node| {
                    for &(label, action, shown) in *section {
                        if shown == Some(MenuButton::Online) {
                            // In a browser there is no server of its own.
                            #[cfg(not(target_arch = "wasm32"))]
                            super::online::spawn_online_row(section_node, gear.clone());
                            continue;
                        }
                        let mut button = spawn_button(
                            section_node,
                            ButtonState::Normal,
                            label,
                            FONT,
                            wide_button(),
                        );
                        button.insert((action, Pressable, action.ui_id("menu")));
                        if let Some(shown) = shown {
                            button.insert(shown);
                        }
                    }
                });
        }
    });
}
