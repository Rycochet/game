//! Builds the dungeon's meshes from what is seen and lit: floors, walls and
//! stairs as textured quads, one mesh per kind of surface (each has its own
//! tile picture), with the light in their vertex colours. What is only
//! remembered goes into a second set of meshes, drawn with grey pictures.

// Split into files (`cargo xtask lint` keeps files small).
mod build;
mod builder;
mod cells;
mod cutaway;
mod flow;
mod light;
mod light_grid;
mod marks;
mod memory;
mod meshes;
mod surface;

#[cfg(test)]
pub use build::build;
pub use build::build_posed;
use build::{RAISED, RAISED_NORMAL};
pub use builder::Builder;
use cells::build_cell_at;
pub(super) use light::corner_light;
pub use light::{faces_camera, with_ambient};
pub use light_grid::LightGrid;
pub(in crate::dungeon_map) use marks::Marks;
use memory::{copy_as_memory, sizes};
pub use meshes::{DungeonMeshes, View, WallFace};
pub use surface::Surface;

/// How bright remembered (out of sight) surfaces are drawn, on their grey
/// pictures.
pub const MEMORY_SHADE: f32 = 0.3;

#[cfg(test)]
mod tests;
