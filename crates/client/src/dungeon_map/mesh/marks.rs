//! What every mesh held before a cell (or a building) was built, so all it
//! built can be draped onto the ground's shape afterwards.

use bevy::prelude::*;

use super::super::walls::Layers;
use super::Builder;

/// How much every mesh held before a cell was built.
pub(in crate::dungeon_map) struct Marks {
    solid: Vec<usize>,
    memory: Vec<usize>,
    see_through: usize,
    stairs_through: Vec<usize>,
    outlined: [usize; 3],
    faces: usize,
}

impl Marks {
    pub(in crate::dungeon_map) fn of(layers: &Layers) -> Self {
        let lengths = |builders: &[super::Builder]| -> Vec<usize> {
            builders.iter().map(|b| b.positions.len()).collect()
        };
        Self {
            solid: lengths(&layers.solid),
            memory: lengths(&layers.memory),
            see_through: layers.see_through.positions.len(),
            stairs_through: lengths(&layers.stairs_through),
            outlined: layers.outlined.sizes(),
            faces: layers.solid_faces.len(),
        }
    }

    /// Raises everything built since onto the ground, each vertex by the
    /// ground's height under it (`ground(position)`, in world units), so
    /// floors follow the slope and upright faces stand on it.
    pub(in crate::dungeon_map) fn drape(&self, layers: &mut Layers, ground: impl Fn(Vec3) -> f32) {
        self.each(layers, |builder, from| builder.drape(from, &ground));
        for face in &mut layers.solid_faces[self.faces..] {
            face.centre.y += ground(face.centre);
        }
    }

    /// Does `change` to each mesh, with where it was before.
    pub(super) fn each(&self, layers: &mut Layers, mut change: impl FnMut(&mut Builder, usize)) {
        for (builder, &from) in layers.solid.iter_mut().zip(&self.solid) {
            change(builder, from);
        }
        for (builder, &from) in layers.memory.iter_mut().zip(&self.memory) {
            change(builder, from);
        }
        change(&mut layers.see_through, self.see_through);
        for (builder, &from) in layers.stairs_through.iter_mut().zip(&self.stairs_through) {
            change(builder, from);
        }
        for (builder, from) in layers.outlined.builders().into_iter().zip(self.outlined) {
            change(builder, from);
        }
    }
}
