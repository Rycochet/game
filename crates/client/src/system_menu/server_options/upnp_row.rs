//! Server Options' router rows (native only): the switch for asking the
//! router to forward the port (UPnP), and a line saying how that went, with a
//! button copying the address players on the internet connect to.
//!
//! Like the boxes above, the switch is kept when Server Options closes, which
//! opens the server again if it is online.

use bevy::prelude::*;
use shared::protocol::{OnlineSettings, OnlineState, RouterPort};

use super::super::{Action, BUTTON_ROW, FONT, GAP, SystemMenu};
use crate::common::{Hosting, Preferences};
use crate::widgets::{
    ButtonPressed, ButtonState, INK, Pressable, Switch, TextSize, UiId, WARNING_TEXT,
    copy_to_clipboard, field_label, spawn_copy_button, spawn_switch,
};

/// The switch.
#[derive(Component)]
pub(in crate::system_menu) struct UpnpSwitch;

/// The line saying how asking the router went.
#[derive(Component)]
pub(in crate::system_menu) struct RouterText;

/// Copy beside it, shown only when there is an address to copy.
#[derive(Component)]
pub(in crate::system_menu) struct RouterCopy;

pub(super) fn spawn_upnp_rows(panel: &mut ChildSpawnerCommands, width: f32) {
    let row = Node {
        align_items: AlignItems::Center,
        column_gap: px(GAP),
        ..default()
    };
    panel.spawn(row.clone()).with_children(|row| {
        row.spawn(field_label("Open the port on my router (UPnP)"));
        spawn_switch(row, ButtonState::Normal, true, default())
            .insert((UpnpSwitch, UiId::new("server_options", "upnp")));
    });
    panel.spawn(row).with_children(|row| {
        row.spawn((
            RouterText,
            Text::new(""),
            TextSize::Small.font(),
            TextColor(INK),
            Node {
                width: px(width - BUTTON_ROW - GAP),
                ..default()
            },
        ));
        let node = Node {
            width: px(BUTTON_ROW),
            height: px(BUTTON_ROW),
            flex_shrink: 0.0,
            ..default()
        };
        spawn_copy_button(row, "Copy the address", FONT, node).insert((
            Action::CopyRouterAddress,
            Action::CopyRouterAddress.ui_id("server_options"),
            Pressable,
            RouterCopy,
            Visibility::Hidden,
        ));
    });
}

/// What the line says, whether it is a warning, and the address to copy.
pub(super) fn router_words(
    hosting: &OnlineState,
    settings: &OnlineSettings,
) -> (String, bool, Option<String>) {
    let port = settings.port;
    let OnlineState::Online { router, .. } = hosting else {
        let words = if settings.upnp {
            "Going online asks the router to forward the port to this computer."
        } else {
            "Players on the internet need the port forwarded to this computer on your router."
        };
        return (words.into(), false, None);
    };
    match router {
        RouterPort::Off => (
            format!("Not asked: forward port {port} to this computer on your router yourself."),
            false,
            None,
        ),
        RouterPort::Opening => (
            format!("Asking the router to forward port {port}..."),
            false,
            None,
        ),
        RouterPort::Open { external, .. } if external.is_empty() => (
            format!("The router forwards port {port} to this computer."),
            false,
            None,
        ),
        RouterPort::Open {
            external,
            shared: false,
        } => (
            format!(
                "The router forwards port {port}: players on the internet connect to {external}."
            ),
            false,
            Some(external.clone()),
        ),
        RouterPort::Open {
            external,
            shared: true,
        } => (
            format!(
                "The router forwards port {port}, but its address {external} is shared by your \
                 internet provider, so players on the internet still can't reach this computer."
            ),
            true,
            None,
        ),
        RouterPort::Failed(reason) => (
            format!(
                "Can't forward the port on the router: {reason}. Forward port {port} yourself."
            ),
            true,
            None,
        ),
    }
}

/// Opening sets the switch from what is kept, closing keeps it; the line
/// follows the server's news, and Copy copies the address.
#[allow(clippy::too_many_arguments)]
pub(in crate::system_menu) fn run_upnp_rows(
    menu: Res<SystemMenu>,
    hosting: Res<Hosting>,
    mut preferences: ResMut<Preferences>,
    mut switch: Single<&mut Switch, With<UpnpSwitch>>,
    mut line: Single<(&mut Text, &mut TextColor), With<RouterText>>,
    mut copy_button: Single<&mut Visibility, With<RouterCopy>>,
    mut pressed: MessageReader<ButtonPressed>,
    actions: Query<&Action>,
    mut last: Local<SystemMenu>,
) {
    let before = std::mem::replace(&mut *last, *menu);
    let open = *menu == SystemMenu::ServerOptions;
    if open && before != SystemMenu::ServerOptions {
        switch.0 = preferences.online_settings.upnp;
    }
    if open && (before != SystemMenu::ServerOptions || hosting.is_changed()) {
        let (words, warning, address) = router_words(&hosting.0, &preferences.online_settings);
        line.0.0 = words;
        line.1.0 = if warning { WARNING_TEXT } else { INK };
        **copy_button = if address.is_some() {
            Visibility::Inherited
        } else {
            Visibility::Hidden
        };
    }
    if open {
        let copy = pressed
            .read()
            .filter_map(|press| actions.get(press.0).ok())
            .any(|action| *action == Action::CopyRouterAddress);
        let (.., address) = router_words(&hosting.0, &preferences.online_settings);
        if copy && let Some(address) = address {
            copy_to_clipboard(&address);
        }
    }
    if before == SystemMenu::ServerOptions && !open && preferences.online_settings.upnp != switch.0
    {
        preferences.online_settings.upnp = switch.0;
    }
}

#[cfg(test)]
mod tests;
