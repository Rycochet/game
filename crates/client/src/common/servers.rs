//! Servers on other computers: the ones connected to before, and the one Play
//! connects to.

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

/// What Play connects to: a server on another computer, or `None` for the usual
/// one (single player natively, the page's server in a browser). Starts as the
/// last one connected to; set by Connect, cleared by Logout.
#[derive(Resource, Clone, Debug, Default, PartialEq, Eq)]
pub struct ChosenServer(pub Option<SavedServer>);

impl ChosenServer {
    /// The last server connected to, from what is saved.
    pub fn from_saved(preferences: &Preferences) -> Self {
        let last = preferences.last_server.as_ref();
        Self(last.map(|address| {
            let saved = preferences.servers.iter().find(|s| s.address == *address);
            saved.cloned().unwrap_or(SavedServer {
                address: address.clone(),
                key: None,
                name: None,
            })
        }))
    }
}

impl Preferences {
    /// After connecting to `server` (or the usual one): it goes to the top of the
    /// list, with its key, and Play connects to it next time.
    pub fn connected_to(&mut self, server: Option<&SavedServer>) {
        let Some(server) = server else {
            self.last_server = None;
            return;
        };
        self.servers.retain(|saved| saved.address != server.address);
        self.servers.insert(0, server.clone());
        self.servers.truncate(MOST_SAVED);
        self.last_server = Some(server.address.clone());
    }

    /// Forgets a server, as the bin beside it in Connect asks.
    pub fn forget_server(&mut self, address: &str) {
        self.servers.retain(|saved| saved.address != address);
        if self.last_server.as_deref() == Some(address) {
            self.last_server = None;
        }
    }
}

#[cfg(test)]
mod tests;
