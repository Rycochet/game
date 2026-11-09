//! A stairs model being built: points, faces and bars.

use bevy::prelude::*;
use std::f32::consts::TAU;

use super::super::CELL_SIZE;
use super::super::mesh::{Builder, Surface};
use super::super::outlined::{OUTLINE_WIDTH, Outlined, add_face, grown_face};
use super::{
    MODEL_DEEPEST_SHADE, PLANK_U, PLANK_V, ROPE_TINT, ROPE_UV, STAIRS_DOWN_DEPTH, facing_shade,
    floor_uv,
};

/// A model being built on its cell: `base` is the middle of its bottom,
/// `descends` and `across` the two ways along the floor.
pub(super) struct Model<'a> {
    pub(super) meshes: &'a mut [Builder],
    /// Where what stands above the floor goes, when see-through.
    pub(super) through: Option<&'a mut [Builder]>,
    /// Where its black outline goes (not while see-through).
    pub(super) outlined: &'a mut Outlined,
    pub(super) lit: Vec3,
    pub(super) base: Vec3,
    /// How high it would be, and the height it is cut off at.
    pub(super) height: f32,
    pub(super) ceiling: f32,
    /// How far apart rungs and knots are, compared with going down.
    pub(super) spacing: f32,
    pub(super) descends: Vec3,
    pub(super) across: Vec3,
}

impl Model<'_> {
    /// The colour at a point: darker the deeper below the floor.
    fn colour_at(&self, point: Vec3) -> Vec3 {
        let depth = (-point.y / STAIRS_DOWN_DEPTH).clamp(0.0, 1.0);
        self.lit * (1.0 + (MODEL_DEEPEST_SHADE - 1.0) * depth)
    }

    /// A point in the model, `(along descends, height, along across)`.
    pub(super) fn at(&self, (along, up, side): (f32, f32, f32)) -> Vec3 {
        self.base + self.descends * along + Vec3::Y * up + self.across * side
    }

    /// A face with four corners going round it, shaded by the way it faces.
    /// Its picture keeps its scale; flat ones line up with the cell's.
    pub(super) fn face(&mut self, surface: Surface, corners: [Vec3; 4], normal: Vec3, tint: Vec3) {
        let uvs = if normal.y.abs() > 0.9 {
            let cell = self.base * Vec3::new(1.0, 0.0, 1.0);
            corners.map(|p| floor_uv(p * Vec3::new(1.0, 0.0, 1.0), cell))
        } else {
            let u = (corners[1].distance(corners[0]) / CELL_SIZE).min(1.0);
            let v = (corners[2].distance(corners[1]) / CELL_SIZE).min(1.0);
            [[0.0, 0.0], [u, 0.0], [u, v], [0.0, v]]
        };
        self.face_with(surface, corners, normal, tint, uvs);
    }

    /// A face as `face`, with its picture coordinates given.
    fn face_with(
        &mut self,
        surface: Surface,
        corners: [Vec3; 4],
        normal: Vec3,
        tint: Vec3,
        uvs: [[f32; 2]; 4],
    ) {
        let shade = facing_shade(normal);
        let colours = corners.map(|p| self.colour_at(p) * shade * tint);
        match self.through.as_deref_mut() {
            Some(through) if corners.iter().any(|p| p.y > 0.01) => {
                through[surface as usize].free(corners, normal, uvs, colours);
            }
            _ => {
                self.meshes[surface as usize].free(corners, normal, uvs, colours);
                let hull = grown_face(corners, normal, OUTLINE_WIDTH);
                add_face(&mut self.outlined.black, hull, normal);
            }
        }
    }

    /// A bar from `from` to `to` with `sides` flat sides and a cap at its
    /// upper end, cut off at the ceiling.
    pub(super) fn bar(
        &mut self,
        surface: Surface,
        (from, to): (Vec3, Vec3),
        radius: f32,
        sides: usize,
        tint: Vec3,
    ) {
        if from.y.min(to.y) >= self.ceiling {
            return;
        }
        let cut = |p: Vec3, other: Vec3| {
            if p.y > self.ceiling {
                other + (p - other) * (self.ceiling - other.y) / (p.y - other.y)
            } else {
                p
            }
        };
        let (from, to) = (cut(from, to), cut(to, from));
        self.whole_bar(surface, (from, to), radius, sides, tint);
    }

    /// A bar as `bar`, not cut off (handrails).
    pub(super) fn whole_bar(
        &mut self,
        surface: Surface,
        (from, to): (Vec3, Vec3),
        radius: f32,
        sides: usize,
        tint: Vec3,
    ) {
        let way = (to - from).normalize_or_zero();
        if way == Vec3::ZERO {
            return;
        }
        let helper = if way.y.abs() > 0.9 { Vec3::X } else { Vec3::Y };
        let right = way.cross(helper).normalize();
        let up = right.cross(way);
        let spoke = |i: usize| {
            let angle = i as f32 / sides as f32 * TAU;
            right * angle.cos() + up * angle.sin()
        };
        for i in 0..sides {
            let (a, b) = (spoke(i), spoke(i + 1));
            let normal = (a + b).normalize();
            let corners = [
                from + a * radius,
                from + b * radius,
                to + b * radius,
                to + a * radius,
            ];
            // Rope is one colour; wood runs along a plank of its picture.
            let uvs = if tint == ROPE_TINT {
                [ROPE_UV; 4]
            } else if surface == Surface::Door {
                let (u0, v0, v1) = (PLANK_U.0, PLANK_V.0, PLANK_V.1);
                let u1 = (u0 + from.distance(to) / CELL_SIZE).min(PLANK_U.1);
                [[u0, v0], [u0, v1], [u1, v1], [u1, v0]]
            } else {
                let across = a.distance(b) * radius / CELL_SIZE;
                let up = (from.distance(to) / CELL_SIZE).min(1.0);
                [[0.0, 0.0], [across, 0.0], [across, up], [0.0, up]]
            };
            self.face_with(surface, corners, normal, tint, uvs);
        }
        let (top, cap) = if to.y >= from.y {
            (to, way)
        } else {
            (from, -way)
        };
        for i in 0..sides {
            let (a, b) = (top + spoke(i) * radius, top + spoke(i + 1) * radius);
            let uv = if tint == ROPE_TINT {
                ROPE_UV
            } else {
                [PLANK_U.0, PLANK_V.0]
            };
            self.face_with(surface, [top, a, b, top], cap, tint, [uv; 4]);
        }
    }
}
