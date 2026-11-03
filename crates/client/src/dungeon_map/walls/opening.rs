//! The hole in the wall of a door or window, and how its lintel or arch is shaded.

use bevy::prelude::*;

use super::super::theme::Shape;
use super::{ARCH_START, DOOR_TOP, HALF_CELL, OPENING_HALF, WINDOW_SILL, WINDOW_TOP};

/// How the underside of a lintel or arch is shaded.
pub(super) const RISING_SHADE: f32 = 0.7;

/// The hole in the wall of a door or window.
pub(super) struct Opening {
    /// The way the wall runs, and the way out of it.
    pub(super) along: Vec3,
    pub(super) out: Vec3,
    pub(super) window: bool,
    pub(super) shape: Shape,
}

impl Opening {
    /// Where the hole starts above the floor.
    pub(super) fn bottom(&self) -> f32 {
        if self.window { WINDOW_SILL } else { 0.0 }
    }

    /// The top of the hole, at a distance from the middle of the cell: flat,
    /// or curved for a natural door.
    pub(super) fn top(&self, a: f32) -> f32 {
        match (self.shape, self.window) {
            (Shape::Natural, false) => {
                let across = (a / OPENING_HALF).clamp(-1.0, 1.0);
                ARCH_START + (DOOR_TOP - ARCH_START) * (1.0 - across * across).sqrt()
            }
            (_, false) => DOOR_TOP,
            (_, true) => WINDOW_TOP,
        }
    }

    /// Where the wall is cut along its length: its ends, the sides of the
    /// hole, and points round an arch.
    pub(super) fn columns(&self) -> Vec<f32> {
        let mut columns = vec![-HALF_CELL];
        if self.shape == Shape::Natural && !self.window {
            const STEPS: usize = 8;
            for i in 0..=STEPS {
                let angle = std::f32::consts::PI * (1.0 - i as f32 / STEPS as f32);
                columns.push(OPENING_HALF * angle.cos());
            }
        } else {
            columns.extend([-OPENING_HALF, OPENING_HALF]);
        }
        columns.push(HALF_CELL);
        columns
    }
}
