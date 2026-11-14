//! How a token's base lies on the ground under it: tilted to the plane that
//! fits the drawn ground best round it, and lifted where the ground would
//! still poke through (owner, 2026-11-14). Beside a building's flat, the
//! ground's triangles slope steeply up to it, and a flat base there had a
//! slice of it hidden. The model standing on the base stays upright.
//!
//! This is the one place bases' tilt is worked out; `place_camera_and_tokens`
//! applies it.

use std::f32::consts::TAU;

use bevy::prelude::*;

/// Points round the rim, and half as many on the ring halfway in, where the
/// ground is sampled.
const RIM_POINTS: usize = 16;

/// How a base lies: its bottom's middle `bottom` high (world units), tilted
/// so its up is `normal`.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BaseTilt {
    pub bottom: f32,
    pub normal: Vec3,
}

impl BaseTilt {
    /// Lying flat at `height`: on flat ground, and on the world map.
    pub fn flat(height: f32) -> Self {
        Self {
            bottom: height,
            normal: Vec3::Y,
        }
    }

    /// Fits a base of `radius` to the ground, given as a height (world units)
    /// at an offset `(x, z)` from the base's middle. The plane of least
    /// squares through the ground at the middle and on two rings; then lifted
    /// until no sampled point of the ground rises more than `sink` into the
    /// base (half its thickness, so its top always shows whole, without
    /// floating needlessly over a hollow). A plane (flat ground, an even
    /// slope) is fitted exactly, with no lift.
    pub fn fit(radius: f32, sink: f32, ground: impl Fn(f32, f32) -> f32) -> Self {
        let mut points = vec![(0.0, 0.0)];
        for (count, reach) in [(RIM_POINTS, radius), (RIM_POINTS / 2, radius / 2.0)] {
            points.extend((0..count).map(|i| {
                let angle = i as f32 * TAU / count as f32;
                (reach * angle.cos(), reach * angle.sin())
            }));
        }
        let samples: Vec<(f32, f32, f32)> = points
            .into_iter()
            .map(|(x, z)| (x, z, ground(x, z)))
            .collect();
        // Rings of evenly spaced points: x and z sum to nothing and are
        // independent, so each slope is found on its own.
        let mean = samples.iter().map(|s| s.2).sum::<f32>() / samples.len() as f32;
        let slope = |along: fn(&(f32, f32, f32)) -> f32| {
            let spread: f32 = samples.iter().map(|s| along(s).powi(2)).sum();
            samples.iter().map(|s| along(s) * (s.2 - mean)).sum::<f32>() / spread
        };
        let (dx, dz) = (slope(|s| s.0), slope(|s| s.1));
        // Lifted till the highest point above the plane is at most `sink` in.
        let lift = samples
            .iter()
            .map(|&(x, z, h)| h - (mean + dx * x + dz * z) - sink)
            .fold(0.0_f32, f32::max);
        Self {
            bottom: mean + lift,
            normal: Vec3::new(-dx, 1.0, -dz).normalize(),
        }
    }

    /// The base's rotation: its up turned to `normal`.
    pub fn rotation(&self) -> Quat {
        Quat::from_rotation_arc(Vec3::Y, self.normal)
    }

    /// From the base's bottom middle to its middle, `half` its thickness up
    /// along its tilt; twice that reaches its top's middle, where a model's
    /// feet go.
    pub fn up(&self, half: f32) -> Vec3 {
        self.normal * half
    }
}

#[cfg(test)]
mod tests;
