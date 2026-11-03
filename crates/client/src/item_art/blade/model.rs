//! The model being built: its triangles by what they are made of (`Stuff`), and the shapes added to it.

use bevy::prelude::*;

use super::ROUND_SIDES;

/// What each piece is made of, as one mesh is made for each.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum Stuff {
    Blade,
    Handle,
    Fittings,
    /// Fullers and engraving, darker than the blade.
    Dark,
    Inlay,
    Cloth,
}

pub(super) const STUFFS: [Stuff; 6] = [
    Stuff::Blade,
    Stuff::Handle,
    Stuff::Fittings,
    Stuff::Dark,
    Stuff::Inlay,
    Stuff::Cloth,
];

pub(super) type Triangle = [Vec3; 3];

/// Triangles for each stuff, each kept facing the way it should.
#[derive(Default)]
pub(super) struct Model {
    pub(super) triangles: [Vec<Triangle>; STUFFS.len()],
}

impl Model {
    fn face(&mut self, stuff: Stuff, a: Vec3, b: Vec3, c: Vec3, out: Vec3) {
        let forward = (b - a).cross(c - a).dot(out) >= 0.0;
        let triangle = if forward { [a, b, c] } else { [a, c, b] };
        self.triangles[stuff as usize].push(triangle);
    }

    fn quad(&mut self, stuff: Stuff, corners: [Vec3; 4], out: Vec3) {
        let [a, b, c, d] = corners;
        self.face(stuff, a, b, c, out);
        self.face(stuff, a, c, d, out);
    }

    /// Skin over rings of equally many points, with the ends closed.
    pub(super) fn loft(&mut self, stuff: Stuff, rings: &[Vec<Vec3>]) {
        let middle = |ring: &[Vec3]| ring.iter().copied().sum::<Vec3>() / ring.len() as f32;
        for pair in rings.windows(2) {
            let centre = (middle(&pair[0]) + middle(&pair[1])) / 2.0;
            let points = pair[0].len();
            for at in 0..points {
                let next = (at + 1) % points;
                let corners = [pair[0][at], pair[0][next], pair[1][next], pair[1][at]];
                let side = corners.iter().copied().sum::<Vec3>() / 4.0;
                self.quad(stuff, corners, side - centre);
            }
        }
        let ends = [
            (rings.first(), rings.get(1)),
            (
                rings.last(),
                rings.len().checked_sub(2).map(|at| &rings[at]),
            ),
        ];
        for (end, inner) in ends {
            let (Some(end), Some(inner)) = (end, inner) else {
                continue;
            };
            let centre = middle(end);
            let out = centre - middle(inner);
            for at in 0..end.len() {
                let next = (at + 1) % end.len();
                self.face(stuff, centre, end[at], end[next], out);
            }
        }
    }

    /// A round thing along a line in the picture's plane: places and radii along it.
    pub(super) fn tube(&mut self, stuff: Stuff, path: &[(Vec2, f32)], depth: f32) {
        let last = path.len().saturating_sub(1);
        let rings: Vec<Vec<Vec3>> = (0..path.len())
            .map(|at| {
                let (place, radius) = path[at];
                let along = (path[(at + 1).min(last)].0 - path[at.saturating_sub(1)].0)
                    .normalize_or(Vec2::Y);
                let across = Vec2::new(-along.y, along.x);
                (0..ROUND_SIDES)
                    .map(|side| {
                        let angle = side as f32 / ROUND_SIDES as f32 * std::f32::consts::TAU;
                        let flat = place + across * angle.cos() * radius;
                        flat.extend(depth + angle.sin() * radius)
                    })
                    .collect()
            })
            .collect();
        self.loft(stuff, &rings);
    }

    /// A straight tapering point.
    pub(super) fn spike(&mut self, stuff: Stuff, from: Vec2, to: Vec2, radius: f32) {
        self.tube(stuff, &[(from, radius), (to, 0.3)], 0.0);
    }

    /// A round plate facing the camera.
    pub(super) fn disc(&mut self, stuff: Stuff, centre: Vec2, radius: f32, thick: f32) {
        let ring = |z: f32| -> Vec<Vec3> {
            (0..16)
                .map(|side| {
                    let angle = side as f32 / 16.0 * std::f32::consts::TAU;
                    (centre + Vec2::from_angle(angle) * radius).extend(z)
                })
                .collect()
        };
        self.loft(stuff, &[ring(-thick), ring(thick)]);
    }

    /// A strip lying on both faces at this depth, from `from` to `to`.
    pub(super) fn ribbon(&mut self, stuff: Stuff, from: Vec2, to: Vec2, half: f32, depth: f32) {
        let across = (to - from).normalize_or(Vec2::Y).perp() * half;
        for side in [depth, -depth] {
            let corners = [from - across, from + across, to + across, to - across];
            self.quad(
                stuff,
                corners.map(|corner| corner.extend(side)),
                Vec3::Z * side,
            );
        }
    }
}
