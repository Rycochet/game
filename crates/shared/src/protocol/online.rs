//! Opening an embedded server to other players: how, and how it went.

use serde::{Deserialize, Serialize};

use super::DEFAULT_PORT;

/// How the embedded server is opened to other players (Server Options).
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
#[serde(default)]
pub struct OnlineSettings {
    /// What the server calls itself.
    pub name: String,
    /// Where other players connect to, as they type it, if given: for its
    /// certificate's names, and later for announcing it.
    pub public_address: String,
    /// Which of this computer's addresses to listen on: `0.0.0.0` for all.
    pub bind_address: String,
    /// The secure port listened on.
    pub port: u16,
    /// The most players at once, the host among them; 0 for no limit.
    pub player_limit: u16,
}

impl Default for OnlineSettings {
    fn default() -> Self {
        Self {
            name: "Hall of Worlds".into(),
            public_address: String::new(),
            bind_address: "0.0.0.0".into(),
            port: DEFAULT_PORT,
            player_limit: 0,
        }
    }
}

/// The secure port's default until 2026-11-07, moved as many other programs
/// use it too.
const OLD_DEFAULT_PORT: u16 = 8081;

impl OnlineSettings {
    /// Settings saved with the old default port take the new one: nobody chose
    /// that port, it was only the default. Any other port is kept.
    pub fn move_off_old_default(&mut self) {
        if self.port == OLD_DEFAULT_PORT {
            self.port = DEFAULT_PORT;
        }
    }
}

/// Whether a server is open to other players.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub enum OnlineState {
    Offline,
    /// Open, with this certificate (PEM) and its key's fingerprint, for
    /// players to check.
    Online {
        certificate: String,
        fingerprint: String,
    },
    /// It couldn't be opened, and why (the port is taken, say).
    Failed(String),
}

#[cfg(test)]
mod tests;
