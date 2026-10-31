//! Game rules and world generation shared by the client and the server.
//!
//! Everything here must be deterministic: the same inputs give bit-identical
//! results on every platform, including WebAssembly. See
//! `docs/context/architecture.md` for the rules.

pub mod climate;
pub mod dungeon;
pub mod movement;
pub mod pathfinding;
pub mod poi;
pub mod protocol;
pub mod river;
pub mod rng;
pub mod showcase;
pub mod site;
pub mod surface;
pub mod vegetation;
pub mod weather;
pub mod world;
