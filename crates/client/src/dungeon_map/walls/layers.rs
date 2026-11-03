//! The layers meshes are built in: solid, see-through, remembered.

use bevy::prelude::*;

use super::super::mesh::{Builder, Surface, WallFace};

/// Which set of meshes something is drawn into.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Layer {
    /// Solid and lit.
    Solid,
    /// Half see-through and lit: faces turned away from the camera.
    SeeThrough,
    /// Grey: out of sight, remembered.
    Memory,
}

/// The meshes being built, by layer and surface.
pub struct Layers {
    pub solid: Vec<Builder>,
    pub memory: Vec<Builder>,
    pub see_through: Builder,
    /// Stairs in the way of a viewer, by surface.
    pub stairs_through: Vec<Builder>,
    /// The solid wall faces, which block clicks.
    pub solid_faces: Vec<WallFace>,
}

impl Layers {
    pub(in crate::dungeon_map) fn builder(
        &mut self,
        layer: Layer,
        surface: Surface,
    ) -> &mut Builder {
        match layer {
            Layer::Solid => &mut self.solid[surface as usize],
            Layer::Memory => &mut self.memory[surface as usize],
            Layer::SeeThrough => &mut self.see_through,
        }
    }
}
