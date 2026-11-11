//! Server Options (native only, from Online's gear): how this game's own server
//! is opened to other players, and its certificate to give them, with a Copy
//! button. What is typed is kept when it closes, and the server opens again
//! with it if it is online. The router's rows are in `upnp_row`, the
//! difficulties' tickboxes in `difficulty_row`.

use bevy::prelude::*;
use shared::protocol::{OnlineSettings, OnlineState, ServerAddress};
use std::net::IpAddr;

mod difficulty_row;
mod upnp_row;

pub(super) use difficulty_row::run_difficulty_boxes;
pub(super) use upnp_row::run_upnp_rows;

use super::{Action, BUTTON_ROW, FONT, Layer, MARGIN, SystemMenu};
use crate::common::{Hosting, Preferences};
use crate::dom::{Template, document};
use crate::html;
use crate::widgets::{
    ButtonPressed, INK, Pressable, TextBox, TextSize, UiId, copy_to_clipboard, spawn_copy_button,
};

const OPTIONS_WIDTH: f32 = 560.0;
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

impl OptionBox {
    const ALL: [Self; 5] = [
        Self::Name,
        Self::PublicAddress,
        Self::BindAddress,
        Self::Port,
        Self::PlayerLimit,
    ];
}

/// Where the certificate is shown.
#[derive(Component)]
pub(super) struct CertificateText;

/// The layer, as markup. The difficulties' tickboxes, the router's rows,
/// the Copy button and the certificate's own words are still made by code,
/// in the places left for them here.
static SERVER_OPTIONS: Template = html!(
    "system_menu.server_options",
    r#"
<form id="server_options">
  <dialog id="layer" title="Server Options" modal closable depth="1" width="560" hidden>
    <div class="tight">
      <label class="field" for="name">Server name</label>
      <input id="name" type="text"/>
    </div>
    <div class="tight">
      <label class="field" for="public_address">
        Public address, for other players to type (host, or host:port)
      </label>
      <input id="public_address" type="text"/>
    </div>
    <div class="tight">
      <label class="field" for="bind_address">
        Listen on (0.0.0.0 for all of this computer's addresses)
      </label>
      <input id="bind_address" type="text"/>
    </div>
    <div class="tight">
      <label class="field" for="port">Port</label>
      <input id="port" type="text" inputmode="numeric"/>
    </div>
    <div class="tight">
      <label class="field" for="player_limit">Player limit (0 for no limit)</label>
      <input id="player_limit" type="text" inputmode="numeric"/>
    </div>
    <div id="difficulties" class="tight"/>
    <div id="router"/>
    <span nowrap valign="middle">
      <label class="field">Certificate, for players who want to check it is you</label>
      <canvas id="copy_holder"/>
    </span>
    <scroll id="certificate" max-height="20%">
      <canvas id="certificate_words"/>
    </scroll>
  </dialog>
</form>
"#
);

pub(super) fn spawn_server_options(commands: &mut Commands) {
    commands.spawn(document(&SERVER_OPTIONS));
}

/// Gives the built layer and its boxes what Server Options' systems know
/// them by, found by their ids, and fills the places left for code.
pub(super) fn bind_server_options(
    mut commands: Commands,
    new: Query<(Entity, &UiId), Added<UiId>>,
) {
    for (entity, id) in &new {
        let ("server_options", name) = id.parts() else {
            continue;
        };
        let mut bound = commands.entity(entity);
        if let Some(which) = OptionBox::ALL
            .into_iter()
            .find(|which| which.ui_name() == name)
        {
            bound.insert(which);
            continue;
        }
        match name {
            "layer" => bound.insert((Layer(&[SystemMenu::ServerOptions]), Action::Back)),
            "close" => bound.insert(Action::Back),
            "difficulties" => bound.with_children(difficulty_row::spawn_difficulty_row),
            "router" => bound.with_children(|rows| {
                upnp_row::spawn_upnp_rows(rows, OPTIONS_WIDTH - 2.0 * MARGIN);
            }),
            "copy_holder" => bound.with_children(|holder| {
                let node = Node {
                    width: px(BUTTON_ROW),
                    height: px(BUTTON_ROW),
                    flex_shrink: 0.0,
                    ..default()
                };
                spawn_copy_button(holder, "Copy the certificate", FONT, node).insert((
                    Action::CopyCertificate,
                    Action::CopyCertificate.ui_id("server_options"),
                    Pressable,
                ));
            }),
            "certificate_words" => bound.with_child((
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
            )),
            _ => continue,
        };
    }
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
