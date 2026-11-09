//! Checking the address in Connect's box while it is typed: once typing has
//! stopped for `TYPING_PAUSE` (or straight away, when the box was filled
//! another way), the address is asked what it is (`connection::ServerProbe`):
//! the port typed, or `DEFAULT_PORT` then `FALLBACK_PORT`. Connect is only
//! enabled once a server this game can play on has answered, and connects to
//! the port that answered.

use bevy::prelude::*;
use shared::protocol::{
    DEFAULT_PORT, FALLBACK_PORT, PROTOCOL_VERSION, ServerAddress, ServerVersion,
};

use super::super::SystemMenu;
use super::AddressBox;
use super::form::{ConnectForm, address_in};
use crate::common::Preferences;
use crate::connection::{ProbeResult, ServerProbe};
use crate::widgets::{TextBox, TextEdited};

/// How long typing must stop before the address is checked, in seconds.
const TYPING_PAUSE: f64 = 0.6;

/// Where the check of the box's address has got.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(super) enum Checked {
    /// Not an address, or waiting for typing to stop.
    #[default]
    Waiting,
    /// Asked, with no answer yet.
    Checking,
    /// A server this game can play on, at this address (with the port that
    /// answered).
    Found {
        address: ServerAddress,
        version: ServerVersion,
    },
    /// Nothing to connect to there, and why.
    Failed(String),
}

/// The check of the address in the box.
#[derive(Resource, Default)]
pub(in crate::system_menu) struct AddressCheck {
    /// What the check is of: the box's text and the key kept for it.
    of: Option<(String, Option<String>)>,
    /// When it last changed, by the real clock.
    changed_at: f64,
    /// Changed by typing (so wait), not by being filled in.
    typed: bool,
    probe: Option<ServerProbe>,
    pub(super) checked: Checked,
}

impl AddressCheck {
    /// The server to connect to and what it said it is, if `text` is the
    /// address found to have one.
    pub(super) fn ready_for(&self, text: &str) -> Option<(&ServerAddress, &ServerVersion)> {
        match (&self.of, &self.checked) {
            (Some((of, _)), Checked::Found { address, version }) if of == text => {
                Some((address, version))
            }
            _ => None,
        }
    }
}

/// Keeps the check up to date with the box, and the box's grey port with what
/// was found.
#[allow(clippy::too_many_arguments)]
pub(in crate::system_menu) fn check_address(
    menu: Res<SystemMenu>,
    time: Res<Time<Real>>,
    form: Res<ConnectForm>,
    preferences: Res<Preferences>,
    mut edited: MessageReader<TextEdited>,
    mut address: Single<(Entity, &mut TextBox), With<AddressBox>>,
    mut check: ResMut<AddressCheck>,
) {
    let typed = edited.read().any(|edit| edit.0 == address.0);
    if !menu.in_connect() {
        // Asked again when Connect opens again.
        *check = AddressCheck::default();
        return;
    }
    let text = address.1.text.clone();
    let key = address_in(&text).and_then(|shown| form.key_for(&shown, &preferences.servers));
    let now = time.elapsed_secs_f64();
    let of = Some((text.clone(), key.clone()));
    if check.of != of {
        *check = AddressCheck {
            of,
            changed_at: now,
            typed,
            ..default()
        };
    }
    let ports = ServerAddress::ports_to_try(&text);
    if check.checked == Checked::Waiting
        && let Some(asked) = ServerAddress::parse(&text)
        && (!check.typed || now - check.changed_at >= TYPING_PAUSE)
    {
        check.probe = Some(ServerProbe::start(&asked, ports.clone(), key));
        check.checked = Checked::Checking;
    }
    if check.checked == Checked::Checking
        && let Some(result) = check.probe.as_ref().and_then(ServerProbe::result)
        && let Some(asked) = ServerAddress::parse(&text)
    {
        check.checked = checked(&asked, &ports, result);
        check.probe = None;
    }
    // The port found, after an address typed without one.
    let suffix = match &check.checked {
        Checked::Found { address, .. } if ports.len() > 1 => format!(":{}", address.port),
        _ => String::new(),
    };
    if address.1.suffix != suffix {
        address.1.suffix = suffix;
    }
}

/// What a probe's result means for Connect.
fn checked(asked: &ServerAddress, ports: &[u16], result: ProbeResult) -> Checked {
    match result {
        ProbeResult::Found { port, version } if version.compatible() => Checked::Found {
            address: asked.at_port(port),
            version,
        },
        ProbeResult::Found { version, .. } => Checked::Failed(other_version(&version)),
        ProbeResult::Refused(reason) => Checked::Failed(reason),
        ProbeResult::NotAServer => Checked::Failed("That isn't a Hall of Worlds server".into()),
        ProbeResult::NoAnswer => Checked::Failed(no_answer(ports)),
    }
}

fn other_version(version: &ServerVersion) -> String {
    if version.protocol > PROTOCOL_VERSION {
        format!(
            "This server runs Hall of Worlds {}, newer than this game: update it to play there",
            version.version
        )
    } else {
        format!(
            "This server runs Hall of Worlds {}, older than this game",
            version.version
        )
    }
}

fn no_answer(ports: &[u16]) -> String {
    let tried = if ports == [DEFAULT_PORT, FALLBACK_PORT] {
        format!("No answer on port {DEFAULT_PORT} or {FALLBACK_PORT}")
    } else {
        let ports: Vec<String> = ports.iter().map(u16::to_string).collect();
        format!("No answer on port {}", ports.join(" or "))
    };
    if cfg!(target_arch = "wasm32") {
        format!(
            "{tried}. A browser only reaches servers whose certificate is from a certificate authority."
        )
    } else {
        tried
    }
}

#[cfg(test)]
mod tests;
