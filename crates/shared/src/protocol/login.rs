//! Logging in and choosing a character: what happens between `Hello` and
//! `Welcome`. See "Accounts" in `docs/context/architecture.md`.

use std::fmt;

use serde::{Deserialize, Serialize};

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

/// Which character. Made by the server, and also the name of their file, so
/// only letters, digits, `-` and `_`. One arriving from a client is only used
/// after it is found among the account's own.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct CharacterId(String);

impl CharacterId {
    pub fn new(id: &str) -> Option<Self> {
        let fine = |c: char| c.is_ascii_alphanumeric() || c == '-' || c == '_';
        (!id.is_empty() && id.len() <= 32 && id.chars().all(fine)).then(|| Self(id.into()))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// A character in the list to choose from.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct CharacterSummary {
    pub id: CharacterId,
    pub name: String,
    /// When last played (seconds since 1970), 0 for never.
    pub last_played: u64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn guest_tokens_read_back_and_never_show_in_logs() {
        let token = GuestToken([0xab; 32]);
        assert_eq!(GuestToken::from_hex(&token.to_hex()), Some(token));
        assert_eq!(GuestToken::from_hex("abcd"), None);
        assert_eq!(GuestToken::from_hex(&"é".repeat(32)), None);
        let shown = format!("{:?}", Credentials::Guest(token));
        assert!(!shown.contains("ab"), "{shown}");
    }

    #[test]
    fn character_ids_are_safe_file_names() {
        assert!(CharacterId::new("player_1").is_some());
        for bad in ["", "../x", "a b", "a/b", "C:"] {
            assert!(CharacterId::new(bad).is_none(), "{bad}");
        }
    }
}
