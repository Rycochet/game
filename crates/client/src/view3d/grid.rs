//! `ViewGrid`: how the current view lays out its grid in 3D.

use bevy::prelude::*;

use crate::common::DrawnGround;

/// How the current view lays out its grid in 3D, set by the view when it
/// becomes current. Grid position `(x, y)` is drawn at
/// `(x × cell_size, floor, y × cell_size)`; the camera looks at the floor and
/// tokens stand on it. `raised`: the wilderness's cells are raised by their
/// ground's shape (`common::DrawnGround`, flat in buildings), and so are the
/// camera and tokens.
#[derive(Resource, Clone, Copy, Debug, PartialEq)]
pub struct ViewGrid {
    pub cell_size: f32,
    pub floor: f32,
    pub raised: bool,
}

impl ViewGrid {
    /// The floor's height at grid position `(x, y)`, in world units.
    pub fn floor_at(&self, heights: &DrawnGround, x: f32, y: f32) -> f32 {
        match self.raised {
            true => self.floor + heights.drawn_between(x, y) * self.cell_size,
            false => self.floor,
        }
    }
}

impl Default for ViewGrid {
    fn default() -> Self {
        Self {
            cell_size: 1.0,
            floor: 0.0,
            raised: false,
        }
    }
}
