//! Servers: the ones on other computers connected to before, the one Connect
//! chose, and whether this computer's own is there.

use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use shared::protocol::{GAME_NAME, ServerAddress};

use super::Preferences;

/// How many servers are remembered; older ones are forgotten.
const MOST_SAVED: usize = 50;

/// A server connected to before, as kept in `Preferences`.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct SavedServer {
    /// As the player typed it (`ServerAddress`'s own way of showing it).
    pub address: String,
    /// The fingerprint of the public key its certificate must have, if the
    /// player pasted one in. Without, any certificate is accepted.
    #[serde(default)]
    pub key: Option<String>,
    /// What it called itself when Connect checked it, if it said.
    #[serde(default)]
    pub name: Option<String>,
}

impl SavedServer {
    pub fn parsed(&self) -> Option<ServerAddress> {
        ServerAddress::parse(&self.address)
    }

    /// What to call it: its name, or its address if it has none or only the
    /// game's own (every server's until its host names it).
    pub fn label(&self) -> &str {
        match self.name.as_deref().map(str::trim) {
            Some(name) if !name.is_empty() && name != GAME_NAME => name,
            _ => &self.address,
        }
    }
}

/// What the game connects to: a server on another computer chosen in Connect,
/// or `None` for this computer's own (single player natively, the page's
/// server in a browser). Set by Connect; cleared on going back to the startup
/// menu, as Continue and Characters are only ever for the computer's own
/// (owner, 2026-11-10).
#[derive(Resource, Clone, Debug, Default, PartialEq, Eq)]
pub struct ChosenServer(pub Option<SavedServer>);

/// Whether this computer's own server is there to play on, for the startup
/// menu's Continue and Characters. Natively it always is (single player, or
/// `HOW_SERVER`). A browser asks the server the page came from first
/// (`connection`): a page hosted without a game server can only Connect.
#[derive(Resource, Clone, Copy, Debug, PartialEq, Eq)]
pub enum HomeServer {
    /// Still asking.
    Checking,
    Found,
    /// Nothing there to play on (a browser only).
    #[cfg_attr(not(target_arch = "wasm32"), allow(dead_code))]
    Missing,
}

impl Default for HomeServer {
    fn default() -> Self {
        if cfg!(target_arch = "wasm32") {
            Self::Checking
        } else {
            Self::Found
        }
    }
}

impl Preferences {
    /// After joining `server`: it goes to the top of Connect's list, with its
    /// key.
    pub fn joined(&mut self, server: &SavedServer) {
        self.servers.retain(|saved| saved.address != server.address);
        self.servers.insert(0, server.clone());
        self.servers.truncate(MOST_SAVED);
    }

    /// Forgets a server, as the bin beside it in Connect asks.
    pub fn forget_server(&mut self, address: &str) {
        self.servers.retain(|saved| saved.address != address);
    }
}

#[cfg(test)]
mod tests;
