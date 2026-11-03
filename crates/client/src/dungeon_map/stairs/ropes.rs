//! Hanging ropes: plain, knotted, and rope ladders.

use bevy::prelude::*;
use std::f32::consts::{PI, TAU};

use super::super::mesh::Surface;
use super::{
    CURL_HEIGHT, CURL_INNER_RADIUS, CURL_PIECES, CURL_RADIUS, CURL_TURNS, KNOT_HALF_HEIGHT,
    KNOT_RADIUS, KNOT_SPACING, LADDER_HALF_WIDTH, Model, PEG_HEIGHT, PEG_RADIUS, ROPE_INSIDE,
    ROPE_OVER_EDGE, ROPE_RADIUS, ROPE_SAG, ROPE_TINT, ROUND_RADIUS, RUNG_SPACING, SLAT_RADIUS,
};

impl Model<'_> {
    /// A rope hanging straight at the side of the hole, `side` across: going
    /// down it comes up over the edge and is tied to a peg in the floor.
    fn hanging_rope(&mut self, side: f32, down: bool) {
        let along = -(ROUND_RADIUS - ROPE_INSIDE);
        let top = self.height;
        let mut points = vec![self.at((along, CURL_HEIGHT, side))];
        if down {
            for (out, up) in ROPE_OVER_EDGE {
                points.push(self.at((-ROUND_RADIUS - out, top + up, side)));
            }
            let [.., (out, _)] = ROPE_OVER_EDGE;
            let foot = self.at((-ROUND_RADIUS - out, top, side));
            let peg = (foot, foot + Vec3::Y * PEG_HEIGHT);
            self.bar(Surface::Door, peg, PEG_RADIUS, 6, Vec3::ONE);
        } else {
            points.push(self.at((along, top, side)));
        }
        for pair in points.windows(2) {
            let bar = (pair[0], pair[1]);
            self.bar(Surface::Door, bar, ROPE_RADIUS, 4, ROPE_TINT);
        }
    }

    /// Two ropes hanging straight, joined by sagging rope or wooden slats.
    pub(super) fn rope_ladder(&mut self, down: bool, slats: bool) {
        let along = -(ROUND_RADIUS - ROPE_INSIDE);
        let width = LADDER_HALF_WIDTH;
        self.hanging_rope(-width, down);
        self.hanging_rope(width, down);
        let top = self.height + if down { ROPE_OVER_EDGE[0].1 } else { 0.0 };
        let mut up = self.spacing * RUNG_SPACING;
        while up < top - self.spacing * RUNG_SPACING / 3.0 {
            if slats {
                let slat = (
                    self.at((along, up, -width - 0.3)),
                    self.at((along, up, width + 0.3)),
                );
                self.bar(Surface::Door, slat, SLAT_RADIUS, 4, Vec3::ONE);
            } else {
                let middle = self.at((along, up - ROPE_SAG, 0.0));
                let sag = [
                    self.at((along, up, -width)),
                    self.at((along, up - ROPE_SAG * 0.75, -width / 2.0)),
                    middle,
                    self.at((along, up - ROPE_SAG * 0.75, width / 2.0)),
                    self.at((along, up, width)),
                ];
                for pair in sag.windows(2) {
                    let bar = (pair[0], pair[1]);
                    self.bar(Surface::Door, bar, ROPE_RADIUS * 0.8, 4, ROPE_TINT);
                }
            }
            up += self.spacing * RUNG_SPACING;
        }
    }

    /// One rope hanging straight, with knots and a curl on the floor.
    pub(super) fn knotted_rope(&mut self, down: bool) {
        let along = -(ROUND_RADIUS - ROPE_INSIDE);
        self.hanging_rope(0.0, down);
        let top = self.height;
        let mut up = self.spacing * KNOT_SPACING;
        while up < top + if down { 1.0 } else { 0.0 } {
            let knot = (
                self.at((along, up - KNOT_HALF_HEIGHT, 0.0)),
                self.at((along, up + KNOT_HALF_HEIGHT, 0.0)),
            );
            self.bar(Surface::Door, knot, KNOT_RADIUS, 6, ROPE_TINT);
            up += self.spacing * KNOT_SPACING;
        }
        // A curl on the floor, winding in from the foot of the rope.
        let centre = along + CURL_RADIUS;
        let curl: Vec<Vec3> = (0..=CURL_PIECES)
            .map(|piece| {
                let done = piece as f32 / CURL_PIECES as f32;
                let r = CURL_RADIUS + (CURL_INNER_RADIUS - CURL_RADIUS) * done;
                let angle = PI + done * CURL_TURNS * TAU;
                self.at((centre + angle.cos() * r, CURL_HEIGHT, angle.sin() * r))
            })
            .collect();
        for pair in curl.windows(2) {
            let bar = (pair[0], pair[1]);
            self.bar(Surface::Door, bar, ROPE_RADIUS, 4, ROPE_TINT);
        }
    }
}
