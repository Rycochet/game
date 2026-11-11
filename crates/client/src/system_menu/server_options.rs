//! Server Options (native only, from Online's gear): how this game's own server
//! is opened to other players, and its certificate to give them, with a Copy
//! button. What is typed is kept when it closes, and the server opens again
//! with it if it is online. The router's rows are in `upnp_row`, the
//! difficulties' tickboxes in `difficulty_row`.

use bevy::prelude::*;
use shared::protocol::{OnlineSettings, OnlineState, ServerAddress};
use std::net::IpAddr;

mod difficulty_row;
mod layer;
mod upnp_row;

pub(super) use difficulty_row::run_difficulty_boxes;
pub(super) use upnp_row::run_upnp_rows;

use super::{ABOVE_MENU_LAYER, Action, BUTTON_ROW, FONT, GAP, MARGIN, SystemMenu};
use crate::common::{Hosting, Preferences};
use crate::widgets::{
    ButtonPressed, INK, Pressable, TextBox, TextSize, UiId, copy_to_clipboard, field_label,
    spawn_copy_button, spawn_scroll_area, spawn_text_box,
};
use layer::{spawn_layer, spawn_title_row};

const OPTIONS_WIDTH: f32 = 560.0;
/// The most of the window's height the certificate takes before it scrolls.
const CERTIFICATE_SHARE: f32 = 20.0;
const NO_CERTIFICATE: &str = "The certificate is made the first time the server goes online.";

/// Which setting a box is for.
#[derive(Component, Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum OptionBox {
    Name,
    PublicAddress,
    BindAddress,
    Port,
    PlayerLimit,
}

impl OptionBox {
    fn ui_name(self) -> &'static str {
        match self {
            Self::Name => "name",
            Self::PublicAddress => "public_address",
            Self::BindAddress => "bind_address",
            Self::Port => "port",
            Self::PlayerLimit => "player_limit",
        }
    }
}

const BOXES: [(OptionBox, &str, bool); 5] = [
    (OptionBox::Name, "Server name", false),
    (
        OptionBox::PublicAddress,
        "Public address, for other players to type (host, or host:port)",
        false,
    ),
    (
        OptionBox::BindAddress,
        "Listen on (0.0.0.0 for all of this computer's addresses)",
        false,
    ),
    (OptionBox::Port, "Port", true),
    (
        OptionBox::PlayerLimit,
        "Player limit (0 for no limit)",
        true,
    ),
];

/// Where the certificate is shown.
#[derive(Component)]
pub(super) struct CertificateText;

pub(super) fn spawn_server_options(commands: &mut Commands) {
    let panel = spawn_layer(
        commands,
        ABOVE_MENU_LAYER,
        &[SystemMenu::ServerOptions],
        px(OPTIONS_WIDTH),
        Val::Auto,
        Node {
            padding: UiRect::all(px(MARGIN)),
            flex_direction: FlexDirection::Column,
            row_gap: px(GAP),
            ..default()
        },
    );
    commands.entity(panel).with_children(|panel| {
        spawn_title_row(panel, "Server Options", "server_options");
        for (which, label, digits_only) in BOXES {
            panel.spawn(field_label(label));
            let text_box = TextBox {
                takes_keys: true,
                digits_only,
                ..default()
            };
            let node = Node {
                width: percent(100),
                height: px(BUTTON_ROW),
                ..default()
            };
            spawn_text_box(panel, text_box, FONT, node)
                .insert((which, UiId::new("server_options", which.ui_name())));
        }
        difficulty_row::spawn_difficulty_row(panel);
        upnp_row::spawn_upnp_rows(panel, OPTIONS_WIDTH - 2.0 * MARGIN);
        // Its words, and the button copying it beside them.
        panel
            .spawn(Node {
                align_items: AlignItems::Center,
                column_gap: px(GAP),
                ..default()
            })
            .with_children(|row| {
                row.spawn(field_label(
                    "Certificate, for players who want to check it is you",
                ));
                let node = Node {
                    width: px(BUTTON_ROW),
                    height: px(BUTTON_ROW),
                    flex_shrink: 0.0,
                    ..default()
                };
                spawn_copy_button(row, "Copy the certificate", FONT, node).insert((
                    Action::CopyCertificate,
                    Action::CopyCertificate.ui_id("server_options"),
                    Pressable,
                ));
            });
        let area = spawn_scroll_area(panel, Val::Vh(CERTIFICATE_SHARE));
        let key = UiId::new("server_options", "certificate");
        panel.commands().entity(area).insert(key);
        panel.commands().entity(area).with_child((
            CertificateText,
            Text::new(NO_CERTIFICATE),
            TextSize::Small.font(),
            TextColor(INK),
            // Its lines are long: they break anywhere rather than run out.
            TextLayout::linebreak(LineBreak::AnyCharacter),
            Node {
                width: px(OPTIONS_WIDTH - 2.0 * MARGIN - 24.0),
                flex_shrink: 0.0,
                ..default()
            },
        ));
    });
}

/// What a box holds for these settings.
fn shown(which: OptionBox, settings: &OnlineSettings) -> String {
    match which {
        OptionBox::Name => settings.name.clone(),
        OptionBox::PublicAddress => settings.public_address.clone(),
        OptionBox::BindAddress => settings.bind_address.clone(),
        OptionBox::Port => settings.port.to_string(),
        OptionBox::PlayerLimit => settings.player_limit.to_string(),
    }
}

/// Puts what a box holds into the settings; `false` (changing nothing) if it
/// isn't right.
fn take(which: OptionBox, text: &str, settings: &mut OnlineSettings) -> bool {
    let text = text.trim();
    match which {
        OptionBox::Name if !text.is_empty() => settings.name = text.to_owned(),
        OptionBox::PublicAddress if text.is_empty() || ServerAddress::parse(text).is_some() => {
            settings.public_address = text.to_owned();
        }
        OptionBox::BindAddress if text.parse::<IpAddr>().is_ok() => {
            settings.bind_address = text.to_owned();
        }
        OptionBox::Port => match text.parse::<u16>() {
            Ok(port) if port > 0 => settings.port = port,
            _ => return false,
        },
        OptionBox::PlayerLimit => match text.parse() {
            Ok(limit) => settings.player_limit = limit,
            Err(_) => return false,
        },
        _ => return false,
    }
    true
}

/// The certificate to show: the one in use, or the one made last time. Its
/// Windows line ends would show as blank lines.
fn certificate(hosting: &Hosting) -> Option<String> {
    let pem = match &hosting.0 {
        OnlineState::Online { certificate, .. } => Some(certificate.clone()),
        _ => {
            let file = server::default_data_folder()
                .join("tls")
                .join("certificate.pem");
            std::fs::read_to_string(file).ok()
        }
    };
    pem.map(|pem| pem.replace("\r\n", "\n"))
}

/// Opening fills the boxes from what is kept; closing keeps what is right in
/// them; while open, a box that isn't right shows it.
#[allow(clippy::too_many_arguments)]
pub(super) fn run_server_options(
    menu: Res<SystemMenu>,
    hosting: Res<Hosting>,
    mut preferences: ResMut<Preferences>,
    mut boxes: Query<(&OptionBox, &mut TextBox)>,
    mut certificate_text: Single<&mut Text, With<CertificateText>>,
    mut pressed: MessageReader<ButtonPressed>,
    actions: Query<&Action>,
    mut last: Local<SystemMenu>,
) {
    let before = std::mem::replace(&mut *last, *menu);
    let open = *menu == SystemMenu::ServerOptions;
    if open && (before != SystemMenu::ServerOptions || hosting.is_changed()) {
        if before != SystemMenu::ServerOptions {
            for (&which, mut text_box) in &mut boxes {
                text_box.text = shown(which, &preferences.online_settings);
                text_box.error = false;
            }
        }
        certificate_text.0 = certificate(&hosting).unwrap_or_else(|| NO_CERTIFICATE.into());
    }
    if open {
        let mut trial = preferences.online_settings.clone();
        for (&which, mut text_box) in &mut boxes {
            let wrong = !take(which, &text_box.text, &mut trial);
            if text_box.error != wrong {
                text_box.error = wrong;
            }
        }
        let copy = pressed
            .read()
            .filter_map(|press| actions.get(press.0).ok())
            .any(|action| *action == Action::CopyCertificate);
        if copy && let Some(text) = certificate(&hosting) {
            copy_to_clipboard(&text);
        }
    }
    if before == SystemMenu::ServerOptions && !open {
        let mut kept = preferences.online_settings.clone();
        for (&which, text_box) in &boxes {
            take(which, &text_box.text, &mut kept);
        }
        if kept != preferences.online_settings {
            preferences.online_settings = kept;
        }
    }
}

#[cfg(test)]
mod tests;
