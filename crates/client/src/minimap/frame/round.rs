//! The minimap's round outline with its ridged border and north notch.

use super::super::canvas::{Canvas, polygon_distance};
use super::{BORDER, LIGHT, NOTCH_BASE, NOTCH_SPREAD, RIDGE_STRENGTH};

/// The ridge's colour at a point: lit on one side going out from its middle,
/// on the other side coming in, so it stands up from the map. `facing` is how
/// much the way out faces the light, from -1 to 1.
pub(super) fn ridge_colour(facing: f32, outer: bool) -> [u8; 3] {
    let shade = 1.0 + RIDGE_STRENGTH * if outer { facing } else { -facing };
    BORDER.map(|channel| (channel * shade).clamp(0.0, 255.0) as u8)
}

/// The minimap's outline around `centre`: a circle of `radius`, with a notch from
/// the inside of its border to `notch` beyond it, towards `north` (a unit vector
/// on the picture).
#[derive(Clone, Copy, Debug)]
pub struct RoundFrame {
    pub centre: (f32, f32),
    pub radius: f32,
    pub border: f32,
    pub notch: f32,
    pub north: (f32, f32),
}

impl RoundFrame {
    pub(super) fn notch_corners(&self) -> [(f32, f32); 3] {
        let (north_x, north_y) = self.north;
        let (across_x, across_y) = (-north_y, north_x);
        let base = self.radius - self.border;
        let half = (self.border + self.notch) * NOTCH_BASE;
        let at = |along: f32, across: f32| {
            (
                self.centre.0 + north_x * along + across_x * across,
                self.centre.1 + north_y * along + across_y * across,
            )
        };
        [
            at(self.radius + self.notch, 0.0),
            at(base, half),
            at(base, -half),
        ]
    }

    /// Distance from the outline, negative inside: the circle and the notch run
    /// smoothly into each other, so the notch grows out of the border.
    pub fn distance(&self, x: f32, y: f32) -> f32 {
        let circle = (x - self.centre.0).hypot(y - self.centre.1) - self.radius;
        // No notch: a plain ring (the Hall of Worlds has no north).
        if self.notch <= 0.0 {
            return circle;
        }
        let notch = polygon_distance(&self.notch_corners(), x, y);
        let spread = (self.border + self.notch).max(0.01) * NOTCH_SPREAD;
        let near = (spread - (circle - notch).abs()).max(0.0) / spread;
        circle.min(notch) - near * near * spread / 4.0
    }

    /// Cuts the picture to the outline and draws the border on it.
    pub fn draw(&self, canvas: &mut Canvas) {
        let inner = self.radius - self.border;
        for y in 0..canvas.height {
            for x in 0..canvas.width {
                let (px, py) = (x as f32 + 0.5, y as f32 + 0.5);
                let from_centre = (px - self.centre.0).hypot(py - self.centre.1);
                // Well inside the border: nothing to do, the usual case.
                if from_centre < inner - 1.0 {
                    continue;
                }
                let distance = self.distance(px, py);
                let at = ((y * canvas.width + x) * 4) as usize;
                if distance >= 0.5 {
                    canvas.pixels[at..at + 4].fill(0);
                    continue;
                }
                // The way out, which round the notch isn't straight from the middle.
                let (out_x, out_y) = (
                    self.distance(px + 0.5, py) - self.distance(px - 0.5, py),
                    self.distance(px, py + 0.5) - self.distance(px, py - 0.5),
                );
                let facing = (out_x * LIGHT.0 + out_y * LIGHT.1) / out_x.hypot(out_y).max(0.01);
                let colour = if distance < -self.border {
                    // Deep in the notch: as deep in the compass.
                    BORDER.map(|channel| channel as u8)
                } else {
                    ridge_colour(facing, distance > -self.border / 2.0)
                };
                canvas.blend(x, y, colour, (from_centre - inner + 0.5).clamp(0.0, 1.0));
                // The outer edge fades out over the picture's transparency.
                if from_centre >= inner {
                    canvas.pixels[at + 3] = ((0.5 - distance).clamp(0.0, 1.0) * 255.0) as u8;
                }
            }
        }
    }
}

/// Distance from a point to a line with rounded ends, `half` to each side of
/// it, negative inside.
pub fn line_distance(from: (f32, f32), to: (f32, f32), half: f32, x: f32, y: f32) -> f32 {
    let (along_x, along_y) = (to.0 - from.0, to.1 - from.1);
    let (at_x, at_y) = (x - from.0, y - from.1);
    let length = along_x * along_x + along_y * along_y;
    let along = ((at_x * along_x + at_y * along_y) / length.max(f32::MIN_POSITIVE)).clamp(0.0, 1.0);
    (at_x - along_x * along).hypot(at_y - along_y * along) - half
}

/// Fills a shape in the ridge's style: `band` wide from its edge, its plain
/// colour deeper in. Which way is out comes from the distance function.
pub(super) fn ridged(
    canvas: &mut Canvas,
    centre: (f32, f32),
    reach: f32,
    band: f32,
    distance: impl Fn(f32, f32) -> f32,
) {
    let low = |at: f32| (at - reach - 1.0).floor().max(0.0) as u32;
    let high = |at: f32, end: u32| ((at + reach + 1.0).ceil().max(0.0) as u32).min(end);
    for y in low(centre.1)..high(centre.1, canvas.height) {
        for x in low(centre.0)..high(centre.0, canvas.width) {
            let (px, py) = (x as f32 + 0.5, y as f32 + 0.5);
            let here = distance(px, py);
            if here >= 0.5 {
                continue;
            }
            let (out_x, out_y) = (
                distance(px + 0.5, py) - distance(px - 0.5, py),
                distance(px, py + 0.5) - distance(px, py - 0.5),
            );
            let facing = (out_x * LIGHT.0 + out_y * LIGHT.1) / out_x.hypot(out_y).max(0.01);
            let colour = if here < -band {
                BORDER.map(|channel| channel as u8)
            } else {
                ridge_colour(facing, here > -band / 2.0)
            };
            canvas.blend(x, y, colour, (0.5 - here).clamp(0.0, 1.0));
        }
    }
}
