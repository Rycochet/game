//! Making ids: the server is the only place they come from.

use std::sync::OnceLock;

use ring::rand::{SecureRandom, SystemRandom};
use shared::id::Id;
use uuid::Uuid;

/// The part of a version 6 UUID that says which machine made it. Normally a
/// network address; random numbers say nothing about the host. New each time the
/// server starts, which is just as good: the time and the node together are what
/// have to differ.
fn node() -> &'static [u8; 6] {
    static NODE: OnceLock<[u8; 6]> = OnceLock::new();
    NODE.get_or_init(|| {
        let mut node = [0; 6];
        SystemRandom::new()
            .fill(&mut node)
            .expect("the system can give random numbers");
        node
    })
}

/// A new id, different from every other.
pub fn new_id() -> Id {
    Id(Uuid::now_v6(node()))
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeSet;

    use super::*;

    #[test]
    fn ids_are_version_6_and_all_different() {
        let ids: BTreeSet<_> = (0..10_000).map(|_| new_id()).collect();
        assert_eq!(ids.len(), 10_000);
        assert!(ids.iter().all(|id| id.0.get_version_num() == 6));
    }

    #[test]
    fn a_save_reads_as_text() {
        let id = new_id();
        let text = ron::to_string(&id).unwrap();
        assert!(text.contains('-'), "{text}");
        assert_eq!(ron::from_str::<Id>(&text).unwrap(), id);
    }
}
