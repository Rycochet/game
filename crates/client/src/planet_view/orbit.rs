//! Where the view's camera is round the globe: turned, tilted and zoomed by the
//! same controls as the maps' camera (`steer.rs`), within the globe's own
//! limits. Plain maths, tested alone.

use std::f32::consts::TAU;

use bevy::prelude::*;

/// Turn or tilt per pixel of mouse movement while right-dragging, as on the
/// maps.
const RADIANS_PER_PIXEL: f32 = 0.005;
/// How far above or below the equator it may look from: nearly over a pole,
/// short of straight down on it, where "up" would be undefined.
const MOST_PITCH_DEGREES: f32 = 80.0;
/// Where it starts: far enough above the equator that the north pole shows.
const HOME_PITCH_DEGREES: f32 = 20.0;
/// Distance multiplier per wheel notch, as on the maps.
const ZOOM_PER_NOTCH: f32 = 0.9;
/// The closest it may come, from the globe's middle, in its radii: the globe
/// then more than fills the picture.
const CLOSEST: f32 = 1.25;

/// The camera's place round the globe.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) struct Orbit {
    /// Turn about the poles' axis, in radians: 0 where it starts, growing as
    /// the camera swings east (anticlockwise seen from the north).
    pub(super) yaw: f32,
    /// Angle above the equator, in radians (below it if negative).
    pub(super) pitch: f32,
    /// Distance as a fraction of the starting one (the furthest out).
    pub(super) zoom: f32,
}

impl Default for Orbit {
    fn default() -> Self {
        Self {
            yaw: 0.0,
            pitch: HOME_PITCH_DEGREES.to_radians(),
            zoom: 1.0,
        }
    }
}

impl Orbit {
    /// Back to where it starts.
    pub(super) fn reset(&mut self) {
        *self = Self::default();
    }

    /// Right-dragging: sideways swings the near side of the globe with the
    /// pointer, so the camera goes the other way; down tilts it to look from
    /// further north, as dragging down looks down more steeply on the maps.
    pub(super) fn turn(&mut self, pixels: Vec2) {
        let most = MOST_PITCH_DEGREES.to_radians();
        self.yaw = (self.yaw - pixels.x * RADIANS_PER_PIXEL).rem_euclid(TAU);
        self.pitch = (self.pitch + pixels.y * RADIANS_PER_PIXEL).clamp(-most, most);
    }

    /// Turns by an angle in radians, clockwise on the screen, as the maps'
    /// keys and twisting fingers do.
    pub(super) fn twist(&mut self, radians: f32) {
        self.yaw = (self.yaw + radians).rem_euclid(TAU);
    }

    /// In (positive) or out by wheel notches, never closer than `CLOSEST`
    /// radii when the start is `start` away, nor further out than the start.
    pub(super) fn zoom_by(&mut self, notches: f32, start: f32) {
        self.zoom = (self.zoom * ZOOM_PER_NOTCH.powf(notches)).clamp(closest(start), 1.0);
    }

    /// In by a pinch: fingers twice as far apart halve the distance.
    pub(super) fn zoom_by_pinch(&mut self, pinch: f32, start: f32) {
        self.zoom = (self.zoom / pinch).clamp(closest(start), 1.0);
    }

    /// The camera's transform, `start` from the globe's middle when not
    /// zoomed, looking at it with north up.
    pub(super) fn transform(&self, start: f32) -> Transform {
        let turn = Quat::from_rotation_y(self.yaw);
        let from = turn * Vec3::new(0.0, self.pitch.sin(), self.pitch.cos());
        Transform::from_translation(from * start * self.zoom).looking_at(Vec3::ZERO, Vec3::Y)
    }
}

/// The least zoom for a camera starting `start` away.
fn closest(start: f32) -> f32 {
    (CLOSEST / start).min(1.0)
}

#[cfg(test)]
mod tests;
