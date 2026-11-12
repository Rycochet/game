//! Game rules and world generation shared by the client and the server.
//!
//! Everything here must be deterministic: the same inputs give bit-identical
//! results on every platform, including WebAssembly. See
//! `docs/context/architecture.md` for the rules.

pub mod ability;
pub mod armour;
pub mod blade;
pub mod body;
pub mod celestial;
pub mod clearance;
pub mod climate;
pub mod combat;
pub mod creature;
pub mod difficulty;
pub mod door;
pub mod dungeon;
pub mod energy;
pub mod footwear;
pub mod ground;
pub mod hall;
pub mod id;
pub mod item;
pub mod item_form;
pub mod item_text;
pub mod magic;
pub mod material;
pub mod mob;
pub mod movement;
pub mod name;
pub mod pathfinding;
pub mod place_name;
pub mod poi;
pub mod protocol;
pub mod relief;
pub mod river;
pub mod rng;
pub mod road;
pub mod role;
pub mod sheet;
pub mod shop;
pub mod site;
pub mod skill;
pub mod spot;
pub mod surface;
pub mod underclothes;
pub mod vegetation;
pub mod vitals;
pub mod wear;
pub mod weather;
pub mod wilderness;
pub mod world;
pub mod world_gate;
