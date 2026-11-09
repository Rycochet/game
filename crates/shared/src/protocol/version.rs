//! What a server says it is, at `VERSION_PATH`: so a client can tell a Hall of
//! Worlds server from any other web server, and whether it speaks the same
//! protocol, before connecting. Sent as JSON, which anything can read.

use serde::{Deserialize, Serialize};

use super::PROTOCOL_VERSION;

/// Where a server answers with its `ServerVersion`, on every port.
pub const VERSION_PATH: &str = "/_version";

/// The name every Hall of Worlds server gives: the magic value that tells it
/// apart from other web servers.
pub const GAME_NAME: &str = "Hall of Worlds";

/// A server's answer at `VERSION_PATH`.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ServerVersion {
    /// Always `GAME_NAME`.
    pub game: String,
    /// The program's version, such as `0.1.0`.
    pub version: String,
    /// Its `PROTOCOL_VERSION`: only the same number can play together.
    pub protocol: u32,
    /// What the server calls itself (`HOW_SERVER_NAME`), if it says: older
    /// servers don't.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
}

impl ServerVersion {
    /// This program's.
    pub fn this() -> Self {
        Self {
            game: GAME_NAME.into(),
            version: env!("CARGO_PKG_VERSION").into(),
            protocol: PROTOCOL_VERSION,
            name: None,
        }
    }

    /// This program's, as a server with this name.
    pub fn named(name: &str) -> Self {
        Self {
            name: Some(name.into()),
            ..Self::this()
        }
    }

    /// As JSON, as a server sends it.
    pub fn to_json(&self) -> String {
        serde_json::to_string(self).expect("plain fields always serialise")
    }

    /// What a web server answered, if it is a Hall of Worlds server's version.
    /// Anything else (another web server's page, broken JSON, another game) is
    /// `None`.
    pub fn from_json(answer: &str) -> Option<Self> {
        let version: Self = serde_json::from_str(answer).ok()?;
        (version.game == GAME_NAME).then_some(version)
    }

    /// Whether a client of this program can play on it.
    pub fn compatible(&self) -> bool {
        self.protocol == PROTOCOL_VERSION
    }
}

#[cfg(test)]
mod tests;
