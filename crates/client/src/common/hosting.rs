//! Opening this game's own server to other players (native only): whether it
//! is, as the server last said.

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
