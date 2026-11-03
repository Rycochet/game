//! Logging in as a guest.

use bevy::prelude::*;
use shared::protocol::{ClientMessage, Credentials, LoginMethod};

use super::{Connecting, ServerConnection};

/// Logs in as a guest, the only way there is so far. `Err` says why it can't.
pub(super) fn log_in(
    connection: &ServerConnection,
    connecting: &mut Connecting,
    methods: &[LoginMethod],
) -> Result<(), String> {
    if !methods.contains(&LoginMethod::Guest) {
        return Err("This server needs an account, which this version can't log in to".into());
    }
    let token = match connecting.guest {
        Some(token) => token,
        None => super::guest::token(&connecting.target)?,
    };
    connecting.guest = Some(token);
    connection
        .link
        .send(&ClientMessage::Login(Credentials::Guest(token)));
    Ok(())
}
