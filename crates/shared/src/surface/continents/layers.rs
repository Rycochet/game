//! Noise summed in layers, each double the detail and half the strength.

use fastnoise_lite::FastNoiseLite;

/// A point in noise space.
pub(super) type Point = (f64, f64, f64);

pub(super) fn scaled((x, y, z): Point, by: f64) -> Point {
    (x * by, y * by, z * by)
}

fn at(noise: &FastNoiseLite, (x, y, z): Point) -> f64 {
    noise.get_noise_3d(x, y, z) as f64
}

/// Layered noise, roughly `-1..1` (most of it well inside), and the value of
/// layer `marked` on its own (0 if there are not that many layers).
pub(super) fn layered(noise: &FastNoiseLite, point: Point, layers: u32, marked: u32) -> (f64, f64) {
    let (mut total, mut strengths, mut mark) = (0.0, 0.0, 0.0);
    for layer in 0..layers {
        let scale = (1u64 << layer) as f64;
        let value = at(noise, scaled(point, scale));
        total += value / scale;
        strengths += 1.0 / scale;
        if layer == marked {
            mark = value;
        }
    }
    (total / strengths.max(1.0), mark)
}

/// Ridged layers, `0..=1`: sharp crests where the noise crosses 0, as
/// mountain chains have, smooth hollows between them.
pub(super) fn ridged(noise: &FastNoiseLite, point: Point, layers: u32) -> f64 {
    let (mut total, mut strengths) = (0.0, 0.0);
    for layer in 0..layers {
        let scale = (1u64 << layer) as f64;
        let crest = 1.0 - at(noise, scaled(point, scale)).abs();
        total += crest * crest / scale;
        strengths += 1.0 / scale;
    }
    if strengths > 0.0 {
        total / strengths
    } else {
        0.0
    }
}

/// `point` pushed about by three other layered noises, so coasts and ranges
/// wander instead of looking like blobs.
pub(super) fn warped(noise: &FastNoiseLite, point: Point, strength: f64) -> Point {
    // Far apart in the same noise, so the three pushes don't follow each other.
    let push = |offset: Point| {
        let moved = (point.0 + offset.0, point.1 + offset.1, point.2 + offset.2);
        layered(noise, moved, 3, u32::MAX).0 * strength
    };
    let (x, y, z) = point;
    (
        x + push((0.0, 0.0, 0.0)),
        y + push((5.2, 1.3, 8.1)),
        z + push((1.7, 9.2, 3.4)),
    )
}
