//! Leaning ladders: plain and log.

use bevy::prelude::*;

use super::super::mesh::Surface;
use super::{
    LADDER_ABOVE_FLOOR, LADDER_HALF_WIDTH, LADDER_POLE_RADIUS, LADDER_RUNG_RADIUS, LEAN_REACH,
    LOG_RADIUS, LOG_RUNG_LENGTH, Model, RUNG_SPACING,
};

impl Model<'_> {
    /// The height of a leaning ladder's top: it comes up above the floor when
    /// going down.
    fn lean_top(&self, down: bool) -> f32 {
        self.height + if down { LADDER_ABOVE_FLOOR } else { 0.0 }
    }

    /// A point on the line of a leaning ladder, from the middle of the bottom
    /// to the edge of the hole, on the side opposite `descends` (where the
    /// floor is at the top), at a height.
    fn lean(&self, height: f32) -> f32 {
        -LEAN_REACH * height / self.height
    }

    /// A ladder leaning from the middle of the bottom to the edge of the hole.
    pub(super) fn ladder(&mut self, down: bool) {
        let top = self.lean_top(down);
        for side in [-LADDER_HALF_WIDTH, LADDER_HALF_WIDTH] {
            let pole = (
                self.at((0.0, 0.0, side)),
                self.at((self.lean(top), top, side)),
            );
            self.bar(Surface::Door, pole, LADDER_POLE_RADIUS, 4, Vec3::ONE);
        }
        let mut up = self.spacing * RUNG_SPACING;
        while up < top - self.spacing * RUNG_SPACING / 2.0 {
            let along = self.lean(up);
            let rung = (
                self.at((along, up, -LADDER_HALF_WIDTH)),
                self.at((along, up, LADDER_HALF_WIDTH)),
            );
            self.bar(Surface::Door, rung, LADDER_RUNG_RADIUS, 4, Vec3::ONE);
            up += self.spacing * RUNG_SPACING;
        }
    }

    /// A thick pole leaning like a ladder, with rungs out of alternate sides.
    pub(super) fn log_ladder(&mut self, down: bool) {
        let top = self.lean_top(down);
        let pole = (
            self.at((0.0, 0.0, 0.0)),
            self.at((self.lean(top), top, 0.0)),
        );
        self.bar(Surface::Door, pole, LOG_RADIUS, 8, Vec3::ONE);
        let mut up = self.spacing * RUNG_SPACING / 2.0;
        let mut side = 1.0;
        while up < top - self.spacing * RUNG_SPACING / 2.0 {
            let along = self.lean(up);
            let rung = (
                self.at((along, up, 0.0)),
                self.at((along, up, side * (LOG_RADIUS + LOG_RUNG_LENGTH))),
            );
            self.bar(Surface::Door, rung, LADDER_RUNG_RADIUS, 4, Vec3::ONE);
            up += self.spacing * RUNG_SPACING / 2.0;
            side = -side;
        }
    }
}
