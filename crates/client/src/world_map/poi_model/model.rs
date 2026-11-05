//! A poi's model being built: triangles, prisms, cones, blocks, houses and battlements.

use bevy::prelude::*;

use super::{DARK, Rgb, rectangle};

/// Triangles with one colour and normal each.
#[derive(Default)]
pub struct PoiModel {
    /// Three corners per triangle, in the order they face outwards.
    pub(super) corners: Vec<[Vec3; 3]>,
    pub(super) colours: Vec<Rgb>,
}

impl PoiModel {
    /// A model of these triangles, each with its corners in the order they
    /// face outwards (as `hall_view::GateModel::triangles` gives them).
    pub fn from_triangles(triangles: impl IntoIterator<Item = ([Vec3; 3], Rgb)>) -> Self {
        let (corners, colours) = triangles.into_iter().unzip();
        Self { corners, colours }
    }

    /// A triangle facing away from `inside`.
    fn triangle(&mut self, a: Vec3, b: Vec3, c: Vec3, inside: Vec3, colour: Rgb) {
        let normal = (b - a).cross(c - a);
        let corners = if normal.dot(a - inside) < 0.0 {
            [a, c, b]
        } else {
            [a, b, c]
        };
        self.corners.push(corners);
        self.colours.push(colour);
    }

    /// Upright walls round `ring` (points on the ground plane, relative to
    /// `at`) from `from` to `to`, with a flat lid.
    pub(super) fn prism(&mut self, at: Vec2, ring: &[Vec2], (from, to): (f32, f32), colour: Rgb) {
        let inside = Vec3::new(at.x, (from + to) / 2.0, at.y);
        let point = |p: Vec2, y: f32| Vec3::new(at.x + p.x, y, at.y + p.y);
        for (i, &p) in ring.iter().enumerate() {
            let q = ring[(i + 1) % ring.len()];
            let (a, b, c, d) = (point(p, from), point(q, from), point(q, to), point(p, to));
            self.triangle(a, b, c, inside, colour);
            self.triangle(a, c, d, inside, colour);
        }
        let top = Vec3::new(at.x, to, at.y);
        for (i, &p) in ring.iter().enumerate() {
            let q = ring[(i + 1) % ring.len()];
            self.triangle(point(p, to), point(q, to), top, inside, colour);
        }
    }

    /// Slopes from `ring` at height `from` up to a point at `from + rise`.
    pub(super) fn cone(&mut self, at: Vec2, ring: &[Vec2], from: f32, rise: f32, colour: Rgb) {
        let inside = Vec3::new(at.x, from, at.y);
        let tip = Vec3::new(at.x, from + rise, at.y);
        let point = |p: Vec2| Vec3::new(at.x + p.x, from, at.y + p.y);
        for (i, &p) in ring.iter().enumerate() {
            let q = ring[(i + 1) % ring.len()];
            self.triangle(point(p), point(q), tip, inside, colour);
        }
    }

    /// A box `half` wide each way, from the ground to `height`.
    pub(super) fn block(&mut self, at: Vec2, half: Vec2, height: f32, colour: Rgb) {
        self.prism(at, &rectangle(half), (0.0, height), colour);
    }

    /// A dark opening on the south face, at `z` in front of `x`.
    pub(super) fn doorway(&mut self, x: f32, z: f32, half_width: f32, height: f32) {
        let inside = Vec3::new(x, height / 2.0, z - 1.0);
        let corner = |dx: f32, y: f32| Vec3::new(x + dx, y, z + 0.05);
        let (a, b) = (corner(-half_width, 0.0), corner(half_width, 0.0));
        let (c, d) = (corner(half_width, height), corner(-half_width, height));
        self.triangle(a, b, c, inside, DARK);
        self.triangle(a, c, d, inside, DARK);
    }

    /// A house with a pointed roof.
    pub(super) fn house(&mut self, at: Vec2, half: Vec2, height: f32, walls: Rgb, roof: Rgb) {
        self.block(at, half, height, walls);
        let eaves = half + Vec2::splat(0.3);
        self.cone(
            at,
            &rectangle(eaves),
            height,
            half.min_element() * 1.2,
            roof,
        );
        self.doorway(at.x, at.y + half.y, 0.3, (height * 0.6).min(1.0));
    }

    /// Battlements round the top of `ring`.
    pub(super) fn battlements(&mut self, at: Vec2, ring: &[Vec2], height: f32, colour: Rgb) {
        for &p in ring {
            let top = (height, height + 0.8);
            self.prism(at + p, &rectangle(Vec2::splat(0.4)), top, colour);
        }
    }
}
