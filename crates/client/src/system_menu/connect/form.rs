//! What Connect's presses do, and what it starts with each time it opens.

use bevy::prelude::*;
use shared::protocol::ServerAddress;

use super::super::icons::ConnectIcons;
use super::super::{Action, SystemMenu};
use super::check::AddressCheck;
use super::popups::{CertificateBox, ForgetWords};
use super::{AddressBox, SUGGESTED_SERVER, ServerList, spawn_server_row};
use crate::common::{ChosenServer, Preferences, SavedServer, Session};
use crate::widgets::{ButtonPressed, TextBox, TextFocus};

/// What Connect has been told since it opened.
#[derive(Resource, Default)]
pub(in crate::system_menu) struct ConnectForm {
    /// A certificate's key pasted in (or `None`, cleared), and the address it is for.
    pasted: Option<(String, Option<String>)>,
    /// The server the requester asks about forgetting.
    forgetting: Option<String>,
}

impl ConnectForm {
    /// The key kept for `address`: pasted in (or cleared) here, otherwise saved.
    pub(super) fn key_for(&self, address: &str, saved: &[SavedServer]) -> Option<String> {
        match &self.pasted {
            Some((pasted_for, key)) if pasted_for == address => key.clone(),
            _ => saved
                .iter()
                .find(|server| server.address == address)
                .and_then(|server| server.key.clone()),
        }
    }
}

/// The address in a box, as it is shown and saved, if it is one.
pub(super) fn address_in(text: &str) -> Option<String> {
    ServerAddress::parse(text).map(|address| address.to_string())
}

/// Each time Connect opens: the box holds the server Play would join, or the
/// latest one, or else `SUGGESTED_SERVER`, and has the keyboard; nothing pasted is kept from before, but
/// for the key just refused.
#[allow(clippy::too_many_arguments, clippy::type_complexity)]
pub(in crate::system_menu) fn open_connect(
    menu: Res<SystemMenu>,
    chosen: Res<ChosenServer>,
    preferences: Res<Preferences>,
    mut form: ResMut<ConnectForm>,
    mut focus: ResMut<TextFocus>,
    mut address: Single<(Entity, &mut TextBox), (With<AddressBox>, Without<CertificateBox>)>,
    mut certificate: Single<&mut TextBox, (With<CertificateBox>, Without<AddressBox>)>,
    mut last: Local<SystemMenu>,
) {
    let before = std::mem::replace(&mut *last, *menu);
    if *menu == SystemMenu::Connect && !before.in_connect() {
        *form = ConnectForm::default();
        // Back from a refusal: the key just tried is still in the padlock.
        if before == SystemMenu::Refused
            && let Some(server) = &chosen.0
        {
            form.pasted = Some((server.address.clone(), server.key.clone()));
        }
        let latest = chosen.0.as_ref().or(preferences.servers.first());
        // With no server known yet, the game's own is suggested.
        address.1.text = latest
            .map(|server| server.address.clone())
            .unwrap_or_else(|| SUGGESTED_SERVER.to_owned());
        focus.0 = Some(address.0);
    }
    if *menu == SystemMenu::Certificate && before == SystemMenu::Connect {
        certificate.text.clear();
        focus.0 = None;
    }
}

/// The remembered servers, drawn again whenever they change.
pub(in crate::system_menu) fn fill_server_list(
    preferences: Res<Preferences>,
    icons: Res<ConnectIcons>,
    list: Single<Entity, With<ServerList>>,
    mut shown: Local<Option<Vec<String>>>,
    mut commands: Commands,
) {
    let addresses: Vec<String> = preferences
        .servers
        .iter()
        .map(|server| server.address.clone())
        .collect();
    if shown.as_ref() == Some(&addresses) {
        return;
    }
    let mut list = commands.entity(*list);
    list.despawn_children();
    list.with_children(|list| {
        for (index, address) in addresses.iter().enumerate() {
            spawn_server_row(list, index, address, icons.bin.clone());
        }
    });
    *shown = Some(addresses);
}

/// Connect's presses: picking and forgetting servers, keeping a certificate, and
/// connecting.
#[allow(clippy::too_many_arguments, clippy::type_complexity)]
pub(in crate::system_menu) fn use_connect(
    mut pressed: MessageReader<ButtonPressed>,
    actions: Query<&Action>,
    mut menu: ResMut<SystemMenu>,
    mut session: ResMut<Session>,
    mut chosen: ResMut<ChosenServer>,
    mut preferences: ResMut<Preferences>,
    mut form: ResMut<ConnectForm>,
    check: Res<AddressCheck>,
    mut address: Single<&mut TextBox, (With<AddressBox>, Without<CertificateBox>)>,
    certificate: Single<&TextBox, (With<CertificateBox>, Without<AddressBox>)>,
    mut forget_words: Single<&mut Text, With<ForgetWords>>,
) {
    let action = pressed
        .read()
        .filter_map(|press| actions.get(press.0).ok())
        .last()
        .copied();
    let (Some(action), true) = (action, menu.in_connect()) else {
        return;
    };
    let in_box = address_in(&address.text);
    match (action, *menu) {
        (Action::PickServer(index), SystemMenu::Connect) => {
            if let Some(server) = preferences.servers.get(index) {
                address.text = server.address.clone();
            }
        }
        (Action::ForgetServer(index), SystemMenu::Connect) => {
            if let Some(server) = preferences.servers.get(index) {
                forget_words.0 = format!("Forget {}?", server.address);
                form.forgetting = Some(server.address.clone());
                *menu = SystemMenu::Forgetting;
            }
        }
        (Action::ReallyForget, SystemMenu::Forgetting) => {
            if let Some(forgotten) = form.forgetting.take() {
                preferences.forget_server(&forgotten);
                if chosen.0.as_ref().is_some_and(|c| c.address == forgotten) {
                    chosen.0 = None;
                }
            }
            *menu = SystemMenu::Connect;
        }
        (Action::Certificate, SystemMenu::Connect) if in_box.is_some() => {
            *menu = SystemMenu::Certificate;
        }
        (Action::KeepCertificate, SystemMenu::Certificate) => {
            if let (Some(address), Some(key)) = (in_box, pasted_key(&certificate.text)) {
                form.pasted = Some((address, Some(key)));
                *menu = SystemMenu::Connect;
            }
        }
        (Action::ClearCertificate, SystemMenu::Certificate) => {
            if let Some(address) = in_box {
                form.pasted = Some((address, None));
            }
            *menu = SystemMenu::Connect;
        }
        (Action::ConnectNow, SystemMenu::Connect) => {
            // The port that answered is always kept with the server (owner).
            let found = check
                .ready_for(&address.text)
                .map(|found| found.with_port());
            if let (Some(typed), Some(found)) = (in_box, found) {
                let key = form.key_for(&typed, &preferences.servers);
                address.text.clone_from(&found);
                chosen.0 = Some(SavedServer {
                    address: found,
                    key,
                });
                *session = Session::Playing;
                *menu = SystemMenu::Closed;
            }
        }
        _ => {}
    }
}

/// The key in what was pasted, if it holds one (native only: in a browser, the
/// browser checks certificates).
pub(super) fn pasted_key(text: &str) -> Option<String> {
    #[cfg(not(target_arch = "wasm32"))]
    return crate::connection::server_key(text);
    #[cfg(target_arch = "wasm32")]
    {
        let _ = text;
        None
    }
}
