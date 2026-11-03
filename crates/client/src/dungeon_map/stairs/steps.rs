//! Solid stairs: spiral and turning, with steps, rails and posts.

use bevy::prelude::*;
use std::f32::consts::{PI, TAU};

use super::super::mesh::Surface;
use super::{
    COLUMN_RADIUS, FLIGHT_STEPS, LANDING_DEPTH, Model, POST_RADIUS, RAIL_HEIGHT, RAIL_RADIUS,
    RIM_POST_EVERY, RIM_RAIL_OUTSIDE, RIM_RAIL_PIECES, RISER_SHADE, ROUND_RADIUS, SPIRAL_GAP,
    SPIRAL_STEP_OVERLAP, SPIRAL_STEP_PIECES, SPIRAL_STEPS, SPIRAL_TURNS, SQUARE_HALF, TREAD_TINT,
    front_tint,
};

impl Model<'_> {
    /// A step of solid stairs: its tread and front (the side facing `front`)
    /// in the floor's picture, its other sides, darker, in the wall's.
    fn steps_block(&mut self, top: Vec3, half: (f32, f32), bottom: f32, front: Vec3) {
        let top = Vec3::new(top.x, top.y.min(self.ceiling), top.z);
        let (d, a) = (self.descends * half.0, self.across * half.1);
        let corners = [top - d - a, top + d - a, top + d + a, top - d + a];
        self.face(Surface::Floor, corners, Vec3::Y, Vec3::splat(TREAD_TINT));
        for (i, normal) in [-self.across, self.descends, self.across, -self.descends]
            .into_iter()
            .enumerate()
        {
            let (p, q) = (corners[i], corners[(i + 1) % 4]);
            let down = |p: Vec3| Vec3::new(p.x, bottom, p.z);
            let side = [p, q, down(q), down(p)];
            if normal == front {
                self.face(Surface::Floor, side, normal, front_tint(normal));
            } else {
                self.face(Surface::Wall, side, normal, Vec3::splat(RISER_SHADE));
            }
        }
        let under = corners.map(|p| Vec3::new(p.x, bottom, p.z));
        self.face(Surface::Wall, under, Vec3::NEG_Y, Vec3::splat(RISER_SHADE));
    }

    /// A rail along `points`, with a second bar half way down to what it
    /// stands on.
    fn rail(&mut self, points: &[Vec3]) {
        let middle = Vec3::Y * RAIL_HEIGHT / 2.0;
        for pair in points.windows(2) {
            let bar = (pair[0], pair[1]);
            self.whole_bar(Surface::Door, bar, RAIL_RADIUS, 4, Vec3::ONE);
            let bar = (pair[0] - middle, pair[1] - middle);
            self.whole_bar(Surface::Door, bar, RAIL_RADIUS, 4, Vec3::ONE);
        }
    }

    /// A post from a step up to the rail above it.
    fn post(&mut self, foot: Vec3) {
        let top = foot + Vec3::Y * RAIL_HEIGHT;
        self.whole_bar(Surface::Door, (foot, top), POST_RADIUS, 4, Vec3::ONE);
    }

    /// Winds up round a column, from near the side `descends` points at to
    /// the opposite side, where the floor is at the top. Going down, the top
    /// step is a landing of twice the angle, centred on that side, and the
    /// only rail runs round the top of the hole beside the landing, then across
    /// its far edge to the column. Going up, it has no rails.
    pub(super) fn spiral(&mut self, down: bool) {
        let radius = ROUND_RADIUS - SPIRAL_GAP;
        let rise = self.height / SPIRAL_STEPS as f32;
        let turned = SPIRAL_TURNS * TAU;
        let (step_angle, first) = if down {
            let angle = turned / (SPIRAL_STEPS + 1) as f32;
            (angle, turned + angle - (SPIRAL_STEPS + 1) as f32 * angle)
        } else {
            (turned / SPIRAL_STEPS as f32, 0.0)
        };
        let (base, descends, across) = (self.base, self.descends, self.across);
        let way = move |angle: f32| descends * angle.cos() + across * angle.sin();
        let point = move |angle: f32, r: f32, up: f32| base + way(angle) * r + Vec3::Y * up;
        let wall_side = RISER_SHADE * Vec3::ONE;
        for step in 0..SPIRAL_STEPS {
            let a = first + step as f32 * step_angle;
            let landing = down && step + 1 == SPIRAL_STEPS;
            let b = a + step_angle * if landing { 2.0 } else { 1.0 };
            let top = (step + 1) as f32 * rise;
            if top > self.ceiling + 0.01 {
                break;
            }
            let low = (top - rise - SPIRAL_STEP_OVERLAP).max(0.0);
            let pieces = SPIRAL_STEP_PIECES * if landing { 2 } else { 1 };
            for piece in 0..pieces {
                let (from, to) = (
                    a + (b - a) * piece as f32 / pieces as f32,
                    a + (b - a) * (piece + 1) as f32 / pieces as f32,
                );
                let tread = [
                    point(from, COLUMN_RADIUS, top),
                    point(from, radius, top),
                    point(to, radius, top),
                    point(to, COLUMN_RADIUS, top),
                ];
                let edge = [
                    point(from, radius, top),
                    point(to, radius, top),
                    point(to, radius, low),
                    point(from, radius, low),
                ];
                let outwards = way((from + to) / 2.0);
                self.face(Surface::Floor, tread, Vec3::Y, Vec3::splat(TREAD_TINT));
                self.face(Surface::Wall, edge, outwards, wall_side);
                let under = tread.map(|p| Vec3::new(p.x, base.y + low, p.z));
                self.face(Surface::Wall, under, Vec3::NEG_Y, wall_side);
            }
            // The front of the step, facing back the way it winds.
            let front = [
                point(a, COLUMN_RADIUS, top),
                point(a, radius, top),
                point(a, radius, low),
                point(a, COLUMN_RADIUS, low),
            ];
            let backwards = way(a - PI / 2.0);
            self.face(Surface::Floor, front, backwards, front_tint(backwards));
            // The back of the step, facing on the way it winds.
            let back = [
                point(b, COLUMN_RADIUS, top),
                point(b, radius, top),
                point(b, radius, low),
                point(b, COLUMN_RADIUS, low),
            ];
            self.face(Surface::Wall, back, way(b + PI / 2.0), wall_side);
        }

        let high = self.height + RAIL_HEIGHT;
        if down {
            let (start, end) = (turned + step_angle, turned - step_angle + TAU);
            let rim_radius = ROUND_RADIUS + RIM_RAIL_OUTSIDE;
            let rim = |piece: usize| {
                let along = piece as f32 / RIM_RAIL_PIECES as f32;
                point(start + (end - start) * along, rim_radius, high)
            };
            let around: Vec<Vec3> = (0..=RIM_RAIL_PIECES).map(rim).collect();
            self.rail(&around);
            for piece in (0..=RIM_RAIL_PIECES).step_by(RIM_POST_EVERY) {
                self.post(rim(piece) - Vec3::Y * RAIL_HEIGHT);
            }
            self.rail(&[rim(0), point(start, COLUMN_RADIUS, high)]);
        }
        let column = (self.at((0.0, 0.0, 0.0)), self.at((0.0, high, 0.0)));
        self.bar(Surface::Wall, column, COLUMN_RADIUS, 12, wall_side);
    }

    /// Two flights side by side, solid down to the ground, turning back at a
    /// landing: both ends are on the side `open` points at (`descends` or
    /// the opposite). Going up, it has no rails; going down, its only rail
    /// runs round the top of the hole, leaving the way onto the top step open.
    pub(super) fn turning(&mut self, open: Vec3, down: bool) {
        // Steps are placed with the model's axes, so flip them to match the
        // way from the open side to the landing.
        let flip = if open == self.descends { -1.0 } else { 1.0 };
        let half = SQUARE_HALF;
        let flight_half = half / 2.0;
        let run = (2.0 * half - LANDING_DEPTH) / FLIGHT_STEPS as f32;
        let levels = 2 * FLIGHT_STEPS + 1;
        let rise = self.height / levels as f32;
        let bottom = self.base.y;
        // `along` is 0 at the open side, 2 × half at the back.
        let spot = |along: f32, up: f32, side: f32| (flip * (along - half), up, side);

        for step in 0..FLIGHT_STEPS {
            let along = (step as f32 + 0.5) * run;
            // First flight on one side, climbing away from the open side.
            let up = (step + 1) as f32 * rise;
            let top = self.at(spot(along, up, flight_half));
            self.steps_block(top, (run / 2.0, flight_half), bottom, open);
            // Second flight on the other, climbing back.
            let up = (levels - step) as f32 * rise;
            let top = self.at(spot(along, up, -flight_half));
            self.steps_block(top, (run / 2.0, flight_half), bottom, -open);
        }
        let landing = (FLIGHT_STEPS + 1) as f32 * rise;
        let top = self.at(spot(2.0 * half - LANDING_DEPTH / 2.0, landing, 0.0));
        self.steps_block(top, (LANDING_DEPTH / 2.0, half), bottom, open);

        let high = RAIL_HEIGHT;
        let top_rail = self.height + high;
        if down {
            let rim = half + RIM_RAIL_OUTSIDE;
            let (near, far) = (-RIM_RAIL_OUTSIDE, 2.0 * half + RIM_RAIL_OUTSIDE);
            let around = [
                spot(near, top_rail, 0.0),
                spot(near, top_rail, rim),
                spot(far, top_rail, rim),
                spot(far, top_rail, -rim),
                spot(near, top_rail, -rim),
            ]
            .map(|p| self.at(p));
            self.rail(&around);
            for point in around {
                self.post(point - Vec3::Y * high);
            }
        }
    }
}
