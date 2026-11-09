//! Outlines for the level's models (stairs, door leaves, secret doors): their
//! black outline, and what shows of them through whatever hides them
//! (`view3d::XrayMaterial`, white). Built with the rest of the level's
//! meshes, into three meshes of their own.

use bevy::prelude::*;

use super::mesh::Builder;

// Split into files (`cargo xtask lint` keeps files small).
mod parts;
mod prism;

pub(super) use parts::{OutlinedMeshes, apply_outlined, spawn_outlined};
pub(super) use prism::{Prism, grow_convex};

/// How far a model's black outline stands out from it, as bags' do.
pub(super) const OUTLINE_WIDTH: f32 = 0.4;

/// What is being built, by what it is drawn with.
#[derive(Default)]
pub struct Outlined {
    /// Black, seen from inside (`cull_mode: Front`): the inverted hull.
    pub(super) black: Builder,
    /// A model's front, drawn faintly where hidden.
    pub(super) fill: Builder,
    /// A model's outline (or the lines it is known by), drawn where hidden.
    pub(super) lines: Builder,
}

impl Outlined {
    /// The builders, for raising everything built since a mark.
    pub(super) fn builders(&mut self) -> [&mut Builder; 3] {
        [&mut self.black, &mut self.fill, &mut self.lines]
    }

    /// Sizes now, for `builders`.
    pub(super) fn sizes(&self) -> [usize; 3] {
        [&self.black, &self.fill, &self.lines].map(|b| b.positions.len())
    }

    /// Into meshes, each `None` if empty.
    pub(super) fn into_meshes(self) -> OutlinedMeshes {
        let mesh = |builder: Builder| (!builder.indices.is_empty()).then(|| builder.into_mesh());
        OutlinedMeshes {
            black: mesh(self.black),
            fill: mesh(self.fill),
            lines: mesh(self.lines),
        }
    }
}

/// Adds a four-cornered face with `normal` as its front: only where it is
/// drawn matters, so no picture or colour.
pub(super) fn add_face(builder: &mut Builder, corners: [Vec3; 4], normal: Vec3) {
    builder.free(corners, normal, [[0.0; 2]; 4], [Vec3::ONE; 4]);
}

/// A face grown outwards by `width`, along its normal and across itself, so
/// faces grown from a closed shape still close round it: for an inverted
/// hull from any model's faces.
pub(super) fn grown_face(corners: [Vec3; 4], normal: Vec3, width: f32) -> [Vec3; 4] {
    let middle = corners.iter().copied().sum::<Vec3>() / 4.0;
    corners.map(|corner| {
        let outwards = (corner - middle).normalize_or_zero();
        corner + normal * width + outwards * width * std::f32::consts::SQRT_2
    })
}
