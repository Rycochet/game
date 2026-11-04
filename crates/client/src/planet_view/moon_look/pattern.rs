//! The grey of a moon's surface at a point, in each style.

use bevy::math::Vec3;
use shared::rng::Seed;

use super::Style;

/// How many craters a cratered moon has, and their sizes (as distances across
/// the unit sphere).
const CRATERS: usize = 40;
const SMALLEST_CRATER: f32 = 0.05;
const LARGEST_CRATER: f32 = 0.4;

/// One style's settings for one moon.
pub(super) struct Pattern {
    style: Style,
    seed: Seed,
    /// Cratered: each crater's middle and size.
    craters: Vec<(Vec3, f32)>,
    /// Banded: how many stripes, and where they start.
    stripes: f32,
    shift: f32,
}

impl Pattern {
    pub(super) fn new(style: Style, seed: Seed) -> Self {
        let mut rng = seed.derive("pattern").rng();
        let mut craters = Vec::new();
        if style == Style::Cratered {
            for _ in 0..CRATERS {
                let point = Vec3::new(
                    rng.range_f64(-1.0, 1.0) as f32,
                    rng.range_f64(-1.0, 1.0) as f32,
                    rng.range_f64(-1.0, 1.0) as f32,
                );
                // Small ones far more often than large.
                let size = rng.next_f64() as f32;
                let size = SMALLEST_CRATER + (LARGEST_CRATER - SMALLEST_CRATER) * size * size;
                craters.push((point.normalize_or(Vec3::Y), size));
            }
        }
        Self {
            style,
            seed,
            craters,
            stripes: rng.range_f64(6.0, 14.0) as f32,
            shift: rng.next_f64() as f32 * 6.0,
        }
    }

    /// The grey at a point of the unit sphere, about `0..1`.
    pub(super) fn grey(&self, point: Vec3) -> f32 {
        let rough = layered(self.seed, point, 3.0, 4);
        match self.style {
            Style::Cratered => {
                let mut grey = 0.5 + 0.2 * (rough - 0.5);
                for &(middle, size) in &self.craters {
                    let along = point.distance(middle) / size;
                    if along < 1.0 {
                        grey -= 0.12 * (1.0 - along * along);
                    }
                    let rim = (along - 1.0) / 0.15;
                    grey += 0.1 * (-rim * rim).exp();
                }
                grey
            }
            Style::Banded => {
                let wavy = point.y + 0.12 * (layered(self.seed, point, 2.0, 3) - 0.5);
                let stripe = (wavy * self.stripes + self.shift).sin();
                let fine = (wavy * self.stripes * 3.1 + self.shift).sin();
                0.55 + 0.18 * stripe + 0.06 * fine + 0.08 * (rough - 0.5)
            }
            Style::CrackedIce => {
                let crack = |purpose: &str, scale: f32| {
                    let line = (layered(self.seed.derive(purpose), point, scale, 3) - 0.5).abs();
                    1.0 - smoothstep(0.0, 0.025, line)
                };
                let cracks = crack("long", 2.5).max(crack("short", 5.0) * 0.7);
                0.85 + 0.08 * (rough - 0.5) - 0.45 * cracks
            }
            Style::Patchy => {
                let patches = layered(self.seed.derive("patches"), point, 1.6, 3);
                0.72 - 0.28 * smoothstep(0.5, 0.58, patches) + 0.1 * (rough - 0.5)
            }
        }
    }
}

fn smoothstep(from: f32, to: f32, value: f32) -> f32 {
    let t = ((value - from) / (to - from)).clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}

/// Layers of value noise, each twice as fine and half as strong, about `0..1`.
fn layered(seed: Seed, point: Vec3, scale: f32, layers: u32) -> f32 {
    let mut total = 0.0;
    let mut weight = 0.0;
    for layer in 0..layers {
        let strength = 0.5f32.powi(layer as i32);
        let fineness = scale * 2f32.powi(layer as i32);
        total += strength * value_noise(seed.derive(&format!("layer {layer}")), point * fineness);
        weight += strength;
    }
    total / weight
}

/// Smooth noise in `0..1`: random values at whole points, blended between.
fn value_noise(seed: Seed, point: Vec3) -> f32 {
    let base = point.floor();
    let part = point - base;
    let ease = part * part * (Vec3::splat(3.0) - 2.0 * part);
    let corner = |dx: i64, dy: i64, dz: i64| {
        let at = [base.x as i64 + dx, base.y as i64 + dy, base.z as i64 + dz];
        (seed.at(&at) >> 40) as f32 / (1u64 << 24) as f32
    };
    let mix = |a: f32, b: f32, t: f32| a + (b - a) * t;
    let x0 = |dy, dz| mix(corner(0, dy, dz), corner(1, dy, dz), ease.x);
    let y0 = |dz| mix(x0(0, dz), x0(1, dz), ease.y);
    mix(y0(0), y0(1), ease.z)
}
