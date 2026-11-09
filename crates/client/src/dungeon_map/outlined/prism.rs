//! A flat shape given thickness (a door's leaf), as faces, and growing a
//! shape for its outline.

use bevy::prelude::*;

/// A convex outline (`profile`, going round either way, in its own flat
/// coordinates) given thickness from `depth.0` to `depth.1`, placed in the
/// world by `place(point, depth)`.
pub(in crate::dungeon_map) struct Prism<F: Fn(Vec2, f32) -> Vec3> {
    pub(in crate::dungeon_map) profile: Vec<Vec2>,
    pub(in crate::dungeon_map) depth: (f32, f32),
    pub(in crate::dungeon_map) place: F,
}

impl<F: Fn(Vec2, f32) -> Vec3> Prism<F> {
    /// Its faces, each four corners and the way out of it: the two flat
    /// sides (as fans, the last corner repeated) and the edges round them.
    pub(in crate::dungeon_map) fn faces(&self) -> Vec<([Vec3; 4], Vec3)> {
        let (near, far) = self.depth;
        let at = |point: Vec2, depth: f32| (self.place)(point, depth);
        let points = &self.profile;
        let count = points.len();
        let middle_flat = points.iter().copied().sum::<Vec2>() / count as f32;
        let middle = at(middle_flat, (near + far) / 2.0);
        let mut faces = Vec::new();
        let mut push = |corners: [Vec3; 4]| {
            let normal = (corners[1] - corners[0])
                .cross(corners[2] - corners[0])
                .normalize_or_zero();
            let centre = corners.iter().copied().sum::<Vec3>() / 4.0;
            let out = if normal.dot(centre - middle) < 0.0 {
                -normal
            } else {
                normal
            };
            faces.push((corners, out));
        };
        for depth in [near, far] {
            for i in 1..count - 1 {
                let fan = [points[0], points[i], points[i + 1], points[i + 1]];
                push(fan.map(|p| at(p, depth)));
            }
        }
        for i in 0..count {
            let (a, b) = (points[i], points[(i + 1) % count]);
            push([at(a, near), at(b, near), at(b, far), at(a, far)]);
        }
        faces
    }
}

/// A convex outline moved out by `width` all round, its sides staying
/// parallel to the original's.
pub(in crate::dungeon_map) fn grow_convex(points: &[Vec2], width: f32) -> Vec<Vec2> {
    let count = points.len();
    let middle = points.iter().copied().sum::<Vec2>() / count as f32;
    // The way out of the side from `a` to `b`.
    let out = |a: Vec2, b: Vec2| {
        let side = (b - a).normalize_or_zero();
        let normal = Vec2::new(side.y, -side.x);
        if normal.dot((a + b) / 2.0 - middle) < 0.0 {
            -normal
        } else {
            normal
        }
    };
    (0..count)
        .map(|i| {
            let before = points[(i + count - 1) % count];
            let point = points[i];
            let after = points[(i + 1) % count];
            let (n1, n2) = (out(before, point), out(point, after));
            // Where the two moved sides meet.
            let sum = n1 + n2;
            let scale = 1.0 + n1.dot(n2);
            if scale < 1e-4 {
                point + n1 * width
            } else {
                point + sum * (width / scale)
            }
        })
        .collect()
}
