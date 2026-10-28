//! Game rules and world generation shared by the client and the server.
//!
//! Everything here must be deterministic: the same inputs give bit-identical
//! results on every platform, including WebAssembly. See
//! `docs/context/architecture.md` for the rules.

pub mod dungeon;
pub mod rng;
pub mod surface;
pub mod world;
