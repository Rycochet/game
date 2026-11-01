//! Ids for things that last: items, and later mobs and anything else needing
//! one. See `docs/context/shared/item.md`.
//!
//! An id is a version 6 UUID, unique whatever it is for and kept for good. Only
//! the server makes them (`server::new_id`): clients are told them, so nothing
//! here needs a clock or randomness, and it stays deterministic.

use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Something's id for good. In a save it reads as the usual text form of a UUID;
/// in a message it is 16 bytes.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct Id(pub Uuid);

#[cfg(test)]
impl Id {
    /// An id for a test, different for each `n`.
    pub fn for_test(n: u128) -> Self {
        Self(Uuid::from_u128(n))
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
