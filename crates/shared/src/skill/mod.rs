//! Skills: what a creature has learned, as a strict tree (Combat › Melee › Longsword ›
//! Longsword), and how much each node has been practised. See
//! `docs/context/shared/skill.md` and the plan in `docs/context/abilities-skills.md`.
//!
//! The tree is a draft written in `branches/`, with some levels filled in from other
//! tables (each weapon of a family, each kind of creature for unarmed fighting). It is
//! built once, the first time it is needed. A node's id is its place in the built
//! tree, which client and server agree on as they are the same version; saves keep
//! the node's path instead, so the tree can change between versions.

mod branches;
mod draft;
mod familiar;
mod id;
mod learned;
mod old;
mod opposition;
mod practice;
mod rust;
mod save_format;
mod saved_node;
mod tree;
mod window;

pub use familiar::{FAMILIAR_HALVING, familiarity, taught};
pub use id::SkillId;
pub use practice::{Skills, VALUE_SCALE, level_practice};
pub use rust::{HOUR, Rust, STEP_EVERY};
pub use tree::key_of;
pub use window::{WINDOW_TICKS, Windows};

#[cfg(test)]
mod rust_tests;
#[cfg(test)]
mod tests;
