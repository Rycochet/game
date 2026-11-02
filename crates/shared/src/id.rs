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
}

/// The usual text form of a UUID, also what files are named by.
impl fmt::Display for Id {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_survives_a_message() {
        let id = Id::for_test(0x1234);
        let bytes = postcard::to_allocvec(&id).unwrap();
        // 16 bytes and their length.
        assert_eq!(bytes.len(), 17);
        assert_eq!(postcard::from_bytes::<Id>(&bytes).unwrap(), id);
    }
}
