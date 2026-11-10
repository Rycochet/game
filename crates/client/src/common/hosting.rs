//! Opening this game's own server to other players (native only): whether it
//! is, as the server last said; and whether a networked server is locked down,
//! for its staff.

use bevy::prelude::*;
use shared::protocol::OnlineState;

/// How opening the built-in server to others went: set by `connection` from the
/// server's answers, read by `system_menu`.
#[derive(Resource, Clone, Debug, PartialEq, Eq)]
pub struct Hosting(pub OnlineState);

impl Default for Hosting {
    fn default() -> Self {
        Self(OnlineState::Offline)
    }
}

/// Whether the networked server being played on is locked down (only its staff
/// may play), as it tells those who may change that
/// (`ServerMessage::Lockdown`); `None` when not told, so their Online isn't
/// shown. Set by `connection`, read by `system_menu`.
#[derive(Resource, Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Lockdown(pub Option<bool>);
