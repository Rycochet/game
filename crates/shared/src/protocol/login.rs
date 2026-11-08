//! Logging in and choosing a character: what happens between `Hello` and
//! `Welcome`. See "Accounts" in `docs/context/architecture.md`.

use std::fmt;

use serde::{Deserialize, Serialize};

use crate::creature::CreatureKind;
use crate::difficulty::Difficulty;
use crate::id::Id;
use crate::sheet::{Appearance, Build};

/// A way of logging in that a server offers.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum LoginMethod {
    /// No login: a token the client makes and keeps, for one character.
    Guest,
    /// Logging in on a provider's website (OIDC), started with
    /// `Credentials::Provider` naming it. `label` is what to call it, such as
    /// "Google".
    Provider { name: String, label: String },
}

/// What a client logs in with.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum Credentials {
    Guest(GuestToken),
    /// The token a server gave in `LoggedIn`, to log in again without the
    /// provider's website until it runs out.
    Session(SessionToken),
    /// On this provider's website (`LoginMethod::Provider` named so): answered
    /// with `OpenLogin`, then `LoggedIn` and `Characters` once done there, or
    /// `LoginFailed`.
    Provider(String),
}

/// A guest's secret: anyone holding it plays that guest. A client keeps one per
/// server, so a server can't use it on another, and servers only store a hash.
#[derive(Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct GuestToken(pub [u8; 32]);

impl GuestToken {
    /// As text, for keeping in the client's own data.
    pub fn to_hex(&self) -> String {
        to_hex(&self.0)
    }

    /// The token `to_hex` wrote, if this is one.
    pub fn from_hex(text: &str) -> Option<Self> {
        from_hex(text).map(Self)
    }
}

/// By hand, so a token never reaches a log.
impl fmt::Debug for GuestToken {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("GuestToken(hidden)")
    }
}

/// A secret the server gives on logging in through a provider, standing for
/// that login until it goes unused for a while. The client keeps it per server,
/// and the server only stores a hash.
#[derive(Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct SessionToken(pub [u8; 32]);

impl SessionToken {
    /// As text, for keeping in the client's own data.
    pub fn to_hex(&self) -> String {
        to_hex(&self.0)
    }

    /// The token `to_hex` wrote, if this is one.
    pub fn from_hex(text: &str) -> Option<Self> {
        from_hex(text).map(Self)
    }
}

/// By hand, so a token never reaches a log.
impl fmt::Debug for SessionToken {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("SessionToken(hidden)")
    }
}

/// The email address a provider vouched for, sent only to its own player for
/// showing in the menus. The server never keeps it.
#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EmailAddress(pub String);

/// By hand, so an address never reaches a log.
impl fmt::Debug for EmailAddress {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("EmailAddress(hidden)")
    }
}

fn to_hex(bytes: &[u8; 32]) -> String {
    bytes.iter().map(|byte| format!("{byte:02x}")).collect()
}

fn from_hex(text: &str) -> Option<[u8; 32]> {
    if text.len() != 64 || !text.is_ascii() {
        return None;
    }
    let mut bytes = [0; 32];
    for (i, byte) in bytes.iter_mut().enumerate() {
        *byte = u8::from_str_radix(&text[i * 2..i * 2 + 2], 16).ok()?;
    }
    Some(bytes)
}

/// Which character. Made by the server, and also the name of their file, which
/// being a UUID keeps safe. One arriving from a client is only used after it is
/// found among the account's own.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct CharacterId(pub Id);

/// What a client asks a new character to be (`ClientMessage::CreateCharacter`).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct NewCharacter {
    pub name: String,
    /// What they are: refused unless it is `CreatureDef::playable`.
    pub kind: CreatureKind,
    /// How they look: refused unless it suits the kind.
    pub appearance: Appearance,
    /// Their triangles: refused unless valid. The server works out the
    /// attributes from them.
    pub build: Build,
    /// How hard the game is for them: the server's nearest allowed one if it
    /// isn't allowed. The only time Ironman can be chosen.
    pub difficulty: Difficulty,
}

#[cfg(test)]
mod tests;
