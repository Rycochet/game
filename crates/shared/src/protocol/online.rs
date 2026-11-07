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
    /// Whether to ask this network's router to forward the port here (UPnP).
    pub upnp: bool,
}

impl Default for OnlineSettings {
    fn default() -> Self {
        Self {
            name: "Hall of Worlds".into(),
            public_address: String::new(),
            bind_address: "0.0.0.0".into(),
            port: DEFAULT_PORT,
            player_limit: 0,
            upnp: true,
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
    /// players to check, and how asking the router to forward the port went.
    Online {
        certificate: String,
        fingerprint: String,
        router: RouterPort,
    },
    /// It couldn't be opened, and why (the port is taken, say).
    Failed(String),
}

/// Asking this network's router to forward the port (UPnP), while online.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub enum RouterPort {
    /// Not asked: turned off in Server Options.
    Off,
    /// Looking for the router, or waiting for its answer.
    Opening,
    /// Forwarded: players outside reach it at `external` (`ip:port`).
    /// `shared` is true when that address is itself a private or shared one
    /// (the internet provider's own network), so players outside still can't.
    Open { external: String, shared: bool },
    /// It couldn't be forwarded, and why.
    Failed(String),
}

#[cfg(test)]
mod tests;
