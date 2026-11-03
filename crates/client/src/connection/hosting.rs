//! Opening the built-in server to other players (native): once playing on it,
//! it is asked to go online with "Online"'s settings, or offline, whenever they
//! differ from what it was last asked.

use bevy::prelude::*;
use shared::protocol::{ClientMessage, OnlineSettings, OnlineState, ServerMessage};

use super::{FromServer, ServerConnection, ToServer};
use crate::common::{Hosting, Preferences, ServerStatus};

pub(super) fn ask_to_host(
    connection: Option<Res<ServerConnection>>,
    status: Res<ServerStatus>,
    preferences: Res<Preferences>,
    mut from_server: MessageReader<FromServer>,
    mut hosting: ResMut<Hosting>,
    mut to_server: MessageWriter<ToServer>,
    mut asked: Local<Option<Option<OnlineSettings>>>,
) {
    let welcomed = from_server
        .read()
        .any(|FromServer(message)| matches!(message, ServerMessage::Welcome { .. }));
    let on_its_own = connection.is_some_and(|connection| connection.local);
    if welcomed || !on_its_own || !status.is_connected() {
        // A server just started, or none of this game's own: nothing asked yet.
        *asked = None;
        if hosting.0 != OnlineState::Offline && !matches!(hosting.0, OnlineState::Failed(_)) {
            hosting.0 = OnlineState::Offline;
        }
        if !welcomed {
            return;
        }
    }
    let wanted = preferences
        .online
        .then(|| preferences.online_settings.clone());
    if asked.as_ref() != Some(&wanted) {
        to_server.write(ToServer(ClientMessage::SetOnline(wanted.clone())));
        *asked = Some(wanted);
    }
}
