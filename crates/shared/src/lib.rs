//! Game rules and world generation shared by the client and the server.
//!
//! Everything here must be deterministic: the same inputs give bit-identical
//! results on every platform, including WebAssembly. See
//! `docs/context/architecture.md` for the rules.

pub mod armour;
pub mod blade;
pub mod body;
pub mod celestial;
pub mod climate;
pub mod creature;
pub mod dungeon;
pub mod energy;
pub mod ground;
pub mod id;
pub mod item;
pub mod item_form;
pub mod material;
pub mod mob;
pub mod movement;
pub mod pathfinding;
pub mod poi;
pub mod protocol;
pub mod river;
pub mod rng;
pub mod sheet;
pub mod shop;
pub mod showcase;
pub mod site;
pub mod surface;
pub mod vegetation;
pub mod wear;
pub mod weather;
pub mod world;
