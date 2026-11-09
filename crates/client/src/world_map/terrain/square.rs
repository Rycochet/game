//! The square window of terrain on show, and corners of clipped triangles.

use bevy::prelude::*;

use super::super::HALF_SIZE;

/// A corner of a triangle being clipped to the window.
#[derive(Clone, Copy)]
pub(super) struct Corner {
    pub(super) position: Vec3,
    pub(super) normal: Vec3,
    pub(super) colour: Vec4,
    pub(super) cover: Vec4,
    /// Distance inside each of the window's four sides (negative outside).
    pub(super) inside: [f32; 4],
}

impl Corner {
    pub(super) fn lerp(self, other: Self, t: f32) -> Self {
        Self {
            position: self.position.lerp(other.position, t),
            normal: self.normal.lerp(other.normal, t),
            colour: self.colour.lerp(other.colour, t),
            cover: self.cover.lerp(other.cover, t),
            inside: std::array::from_fn(|k| {
                self.inside[k] + (other.inside[k] - self.inside[k]) * t
            }),
        }
    }
}

/// The square window of terrain on show, centred on the player and lined up
/// with the camera, in grid units relative to the centre.
#[derive(Clone, Copy, Debug)]
pub struct ViewSquare {
    /// The camera's right and towards-the-camera directions along the ground,
    /// as (x, z).
    pub right: Vec2,
    pub towards: Vec2,
    /// From the centre to each side: `HALF_SIZE` in the starting view, less
    /// when zoomed in, never more than `MAX_HALF` (see `window_half`).
    pub half: f32,
}

impl ViewSquare {
    /// How far `point` is inside each side: right, left, near and far
    /// (negative outside).
    pub(super) fn inside(&self, point: Vec2) -> [f32; 4] {
        let (u, v) = (point.dot(self.right), point.dot(self.towards));
        [self.half - u, self.half + u, self.half - v, self.half + v]
    }

    /// Whether `point` is on or inside the square.
    pub fn contains(&self, point: Vec2) -> bool {
        self.inside(point).iter().all(|&d| d >= 0.0)
    }

    /// The point `u` halves to the right and `v` halves towards the camera.
    pub(super) fn at(&self, u: f32, v: f32) -> Vec2 {
        (self.right * u + self.towards * v) * self.half
    }

    /// How much of the starting view's width is on show (1 = all of it).
    pub(super) fn zoom(&self) -> f32 {
        self.half / HALF_SIZE as f32
    }
}
