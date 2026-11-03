//! Logging in and choosing a character: what happens between `Hello` and
//! `Welcome`. See "Accounts" in `docs/context/architecture.md`.

use std::fmt;

use serde::{Deserialize, Serialize};

use crate::id::Id;

/// A way of logging in that a server offers.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum LoginMethod {
    /// No login: a token the client makes and keeps, for one character.
    Guest,
}

/// What a client logs in with.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum Credentials {
    Guest(GuestToken),
}

/// A guest's secret: anyone holding it plays that guest. A client keeps one per
/// server, so a server can't use it on another, and servers only store a hash.
#[derive(Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct GuestToken(pub [u8; 32]);

impl GuestToken {
    /// As text, for keeping in the client's own data.
    pub fn to_hex(&self) -> String {
        self.0.iter().map(|byte| format!("{byte:02x}")).collect()
    }

    /// The token `to_hex` wrote, if this is one.
    pub fn from_hex(text: &str) -> Option<Self> {
        if text.len() != 64 || !text.is_ascii() {
            return None;
        }
        let mut token = [0; 32];
        for (i, byte) in token.iter_mut().enumerate() {
            *byte = u8::from_str_radix(&text[i * 2..i * 2 + 2], 16).ok()?;
        }
        Some(Self(token))
    }
}

/// By hand, so a token never reaches a log.
impl fmt::Debug for GuestToken {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("GuestToken(hidden)")
    }
}

/// Which character. Made by the server, and also the name of their file, which
/// being a UUID keeps safe. One arriving from a client is only used after it is
/// found among the account's own.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct CharacterId(pub Id);

/// A character in the list to choose from.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct CharacterSummary {
    pub id: CharacterId,
    pub name: String,
    /// When last played (seconds since 1970), 0 for never.
    pub last_played: u64,
}

#[cfg(test)]
mod tests;
