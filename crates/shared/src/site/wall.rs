//! Walls round sites (owner's design, 2026-11-13): a ring one cell thick round
//! a rectangle, of one material, with gates of two door cells side by side.
//! Forts have them now; castles and walled towns are to use the same pieces.
//!
//! A wall is plain `Cell::Wall` for the rules (it blocks walking and sight
//! like any wall), and its gates are plain `Cell::Door`s, so nothing else needs
//! to know about it. What it is made of and where it runs is told to players
//! as a `WallLook`, with the level, for drawing.

// Split into files (`cargo xtask lint` keeps files small).
mod build;
mod look;
mod material;
mod options;
mod poi_wall;

pub(super) use build::build_wall;
pub use look::{Gate, WallLook};
pub use material::WallMaterial;
pub use options::WallOptions;
pub use poi_wall::PoiWall;

#[cfg(test)]
mod tests;
