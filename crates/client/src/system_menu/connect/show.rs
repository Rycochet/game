//! Keeping Connect up to date as things are typed: Connect only works once a
//! server has been found at the address (`check.rs`), the padlock only for an
//! address, the padlock is shut while a certificate is kept for it,
//! and the certificate popup's Keep only works for a certificate.

use bevy::prelude::*;

use super::super::SystemMenu;
use super::super::icons::ConnectIcons;
use super::check::AddressCheck;
use super::form::{ConnectForm, address_in, pasted_key};
use super::popups::{CertificateBox, KeepButton};
use super::{AddressBox, ConnectNowButton, PadlockButton};
use crate::common::Preferences;
use crate::dom::Dom;
use crate::widgets::{ButtonState, ButtonTooltip, TextBox};

fn enabled_if(usable: bool) -> ButtonState {
    if usable {
        ButtonState::Normal
    } else {
        ButtonState::Disabled
    }
}

#[allow(clippy::too_many_arguments, clippy::type_complexity)]
pub(in crate::system_menu) fn show_connect(
    menu: Res<SystemMenu>,
    form: Res<ConnectForm>,
    check: Res<AddressCheck>,
    preferences: Res<Preferences>,
    icons: Res<ConnectIcons>,
    address: Single<&TextBox, (With<AddressBox>, Without<CertificateBox>)>,
    certificate: Single<&TextBox, (With<CertificateBox>, Without<AddressBox>)>,
    mut connect: Query<
        &mut ButtonState,
        (
            With<ConnectNowButton>,
            Without<PadlockButton>,
            Without<KeepButton>,
        ),
    >,
    mut keep: Query<
        &mut ButtonState,
        (
            With<KeepButton>,
            Without<PadlockButton>,
            Without<ConnectNowButton>,
        ),
    >,
    mut padlock: Query<(&mut ButtonState, &mut ButtonTooltip, &Children), With<PadlockButton>>,
    mut pictures: Query<&mut ImageNode>,
    mut dom: Dom,
    mut status: Local<String>,
) {
    if !menu.in_connect() {
        return;
    }
    let in_box = address_in(&address.text);
    let found = check.ready_for(&address.text).is_some();
    for mut state in &mut connect {
        state.set_if_neq(enabled_if(found));
    }
    let key = in_box
        .as_ref()
        .and_then(|address| form.key_for(address, &preferences.servers));
    for (mut state, mut tooltip, children) in &mut padlock {
        state.set_if_neq(enabled_if(in_box.is_some()));
        let words = if key.is_some() {
            "Certificate kept: only a server with it is accepted"
        } else {
            "Certificate: paste in the server's, so only it is accepted"
        };
        if tooltip.0 != words {
            tooltip.0 = words.into();
        }
        let picture = if key.is_some() {
            &icons.shut
        } else {
            &icons.open
        };
        for &child in children {
            if let Ok(mut image) = pictures.get_mut(child)
                && image.image != *picture
            {
                image.image = picture.clone();
            }
        }
    }
    let pasted = pasted_key(&certificate.text);
    for mut state in &mut keep {
        state.set_if_neq(enabled_if(pasted.is_some()));
    }
    let words = match (&pasted, certificate.text.trim().is_empty()) {
        (Some(key), _) => format!("The key's fingerprint begins {}…", &key[..16]),
        (None, true) => "Paste with Ctrl+V.".to_owned(),
        (None, false) => "That isn't a certificate or a fingerprint.".to_owned(),
    };
    if *status != words {
        dom.set_text("connect.certificate/status", words.clone());
        *status = words;
    }
}
