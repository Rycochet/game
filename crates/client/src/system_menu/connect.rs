//! Connect, from the startup menu: a box for a server's address, the servers
//! connected to before (click one to put it in the box, or bin it), and, on
//! native, a padlock to paste in the server's certificate so that only it is
//! accepted. Connect's own Connect plays there, once `check.rs` has found a
//! server at the address. See `form.rs` for what the presses do.

use bevy::prelude::*;

use super::icons::ConnectIcons;
use super::spawn::{spawn_layer, spawn_title_row};
use super::{ABOVE_MENU_LAYER, Action, BUTTON_ROW, FONT, GAP, MARGIN, SystemMenu};
use crate::widgets::{
    ButtonState, ButtonStyle, ButtonTooltip, INK, Pressable, ScrollKey, TextBox, TextSize,
    spawn_button, spawn_icon_button, spawn_scroll_area, spawn_text_box,
};

// Split into files (`cargo xtask lint` keeps files small).
mod check;
mod form;
mod mark;
mod popups;
mod show;

pub(super) use check::{AddressCheck, check_address};
pub(super) use form::{ConnectForm, fill_server_list, open_connect, use_connect};
pub(super) use mark::show_mark;
pub(super) use popups::spawn_popups;
pub(super) use show::show_connect;

const CONNECT_WIDTH: f32 = 520.0;
/// The most of the window's height the list takes before it scrolls.
const LIST_SHARE: f32 = 40.0;
const EXAMPLE: &str = "For example: play.example.com, or play.example.com:8081";
#[cfg(target_arch = "wasm32")]
const BROWSER_NOTE: &str =
    "A browser only connects to servers whose certificate is from a certificate authority.";

/// The box the address is typed into.
#[derive(Component)]
pub(super) struct AddressBox;

/// The padlock beside it, shut while a certificate is kept for the address.
#[derive(Component)]
pub(super) struct PadlockButton;

/// Connect's own Connect button, only enabled for an address.
#[derive(Component)]
pub(super) struct ConnectNowButton;

/// Where the remembered servers go.
#[derive(Component)]
pub(super) struct ServerList;

pub(super) fn spawn_connect(commands: &mut Commands, icons: &ConnectIcons) {
    let panel = spawn_layer(
        commands,
        ABOVE_MENU_LAYER,
        &[
            SystemMenu::Connect,
            SystemMenu::Certificate,
            SystemMenu::Forgetting,
        ],
        px(CONNECT_WIDTH),
        Val::Auto,
        Node {
            padding: UiRect::all(px(MARGIN)),
            flex_direction: FlexDirection::Column,
            row_gap: px(GAP),
            ..default()
        },
    );
    let icons = icons.clone();
    commands.entity(panel).with_children(|panel| {
        spawn_title_row(panel, "Connect");
        panel.spawn((Text::new(EXAMPLE), TextSize::Medium.font(), TextColor(INK)));
        #[cfg(target_arch = "wasm32")]
        panel.spawn((
            Text::new(BROWSER_NOTE),
            TextSize::Small.font(),
            TextColor(INK),
        ));
        panel
            .spawn(Node {
                column_gap: px(GAP),
                ..default()
            })
            .with_children(|row| {
                let node = Node {
                    flex_grow: 1.0,
                    height: px(BUTTON_ROW),
                    ..default()
                };
                let text_box = TextBox {
                    takes_keys: true,
                    ..default()
                };
                spawn_text_box(row, text_box, FONT, node)
                    .insert(AddressBox)
                    .with_children(|address_box| mark::spawn_mark(address_box, BUTTON_ROW))
                    .entry::<Node>()
                    .and_modify(|mut node| node.padding.right = px(mark::MARK_ROOM));
                let node = Node {
                    width: px(BUTTON_ROW),
                    height: px(BUTTON_ROW),
                    flex_shrink: 0.0,
                    // In a browser the browser checks certificates.
                    display: if cfg!(target_arch = "wasm32") {
                        Display::None
                    } else {
                        Display::Flex
                    },
                    ..default()
                };
                spawn_icon_button(row, ButtonState::Normal, icons.open.clone(), node).insert((
                    PadlockButton,
                    Action::Certificate,
                    Pressable,
                    ButtonTooltip("Certificate".into()),
                ));
            });
        let list = spawn_scroll_area(panel, Val::Vh(LIST_SHARE));
        let key = ScrollKey::new("connect.servers");
        panel.commands().entity(list).insert((ServerList, key));
        panel
            .spawn(Node {
                justify_content: JustifyContent::End,
                column_gap: px(GAP),
                ..default()
            })
            .with_children(|row| {
                for (label, action, style) in [
                    ("Connect", Action::ConnectNow, ButtonStyle::Main),
                    ("Cancel", Action::Back, ButtonStyle::Plain),
                ] {
                    let node = Node {
                        width: px(CONNECT_WIDTH / 4.0),
                        height: px(BUTTON_ROW),
                        ..default()
                    };
                    let mut button = spawn_button(row, ButtonState::Normal, label, FONT, node);
                    button.insert((action, style, Pressable));
                    if action == Action::ConnectNow {
                        button.insert(ConnectNowButton);
                    }
                }
            });
    });
}

/// One remembered server in the list: its address, which puts it in the box, and
/// a bin to forget it.
pub(super) fn spawn_server_row(
    list: &mut ChildSpawnerCommands,
    index: usize,
    address: &str,
    bin: Handle<Image>,
) {
    list.spawn(Node {
        column_gap: px(GAP),
        margin: UiRect::bottom(px(GAP / 2.0)),
        flex_shrink: 0.0,
        width: px(CONNECT_WIDTH - 2.0 * MARGIN - 24.0),
        ..default()
    })
    .with_children(|row| {
        let node = Node {
            flex_grow: 1.0,
            height: px(BUTTON_ROW),
            ..default()
        };
        spawn_button(row, ButtonState::Normal, address, FONT, node)
            .insert((Action::PickServer(index), Pressable));
        let node = Node {
            width: px(BUTTON_ROW),
            height: px(BUTTON_ROW),
            flex_shrink: 0.0,
            ..default()
        };
        spawn_icon_button(row, ButtonState::Normal, bin, node).insert((
            Action::ForgetServer(index),
            Pressable,
            ButtonTooltip("Forget this server".into()),
        ));
    });
}
