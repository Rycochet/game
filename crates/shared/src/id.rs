//! Ids for things that last: items, and later mobs and anything else needing
//! one. See `docs/context/shared/item.md`.
//!
//! An id is a version 6 UUID, unique whatever it is for and kept for good. Only
//! the server makes them (`server::new_id`): clients are told them, so nothing
//! here needs a clock or randomness, and it stays deterministic.

use std::fmt;

use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::rng::Seed;

/// Something's id for good. In a save it reads as the usual text form of a UUID;
/// in a message it is 16 bytes.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct Id(pub Uuid);

impl Id {
    /// Stands for no one: no real id is nil.
    pub const NOBODY: Id = Id(Uuid::nil());

    /// A seed only this thing has, for what is rolled when it is made. The two halves
    /// are mixed as the low one alone hardly changes from one id to the next.
    pub fn seed(self) -> Seed {
        let bits = self.0.as_u128();
        Seed::new((bits >> 64) as u64 ^ bits as u64)
    }

    /// An id for a test in any crate, different for each `n`. The game's own come
    /// from `server::new_id`.
    pub fn for_test(n: u128) -> Self {
        Self(Uuid::from_u128(n))
    }

    /// A stand-in, never a real id: what a guess names something new by until the
    /// server's answer, with the real id, replaces it. A version 8 UUID, so it can't
    /// be one the server made (version 6), different for each `n`.
    pub fn stand_in(n: u64) -> Self {
        let mut bytes = [0; 16];
        bytes[8..].copy_from_slice(&n.to_be_bytes());
        Self(uuid::Builder::from_custom_bytes(bytes).into_uuid())
    }

    /// Whether it is a stand-in (`stand_in`).
    pub fn is_stand_in(self) -> bool {
        self.0.get_version_num() == 8
    }

    /// The id written as `Display` writes it (hyphenated), if `text` is one.
    pub fn parse(text: &str) -> Option<Self> {
        // `parse_str` also takes other forms (simple, braced, urn): only this one.
        let hyphenated = text.len() == 36
            && text.char_indices().all(|(at, c)| match at {
                8 | 13 | 18 | 23 => c == '-',
                _ => c.is_ascii_hexdigit(),
            });
        Uuid::parse_str(text).ok().filter(|_| hyphenated).map(Self)
    }
}

/// Stand-ins one after another (`Id::stand_in`), from 1: for a trial or a test,
/// where nothing they name is kept.
pub fn stand_ins() -> impl FnMut() -> Id {
    let mut n = 0;
    move || {
        n += 1;
        Id::stand_in(n)
    }
}

/// The usual text form of a UUID, also what files are named by.
impl fmt::Display for Id {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

#[cfg(test)]
mod tests;
