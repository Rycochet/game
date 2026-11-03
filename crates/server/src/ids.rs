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
mod tests;
