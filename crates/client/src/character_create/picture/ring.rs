//! The ring round the triangles, and a curved arrow pointing both ways in each
//! gap between them, saying they can be turned.

use bevy::math::Vec2;

use super::super::geometry::{MIDDLE, distance_to_shape, gaps, round_middle};
use super::{blend, cover};

/// How far the ring is from the middle, and half its width, as shares of the
/// picture: just outside the triangles' furthest corners.
pub(super) const RING_RADIUS: f32 = 0.485;
const RING_HALF: f32 = 0.006;
/// The arrows run just inside the ring.
const ARROW_RADIUS: f32 = 0.44;
const ARROW_HALF: f32 = 0.0045;
/// How far each arrow reaches either side of its gap, in degrees, heads included.
const ARROW_SPREAD: f32 = 17.0;
/// Each head's length along the curve, in degrees, and half its width.
const HEAD_LENGTH: f32 = 5.5;
const HEAD_HALF: f32 = 0.016;

/// A button's colour (`widgets` `RAISED_BACKGROUND`), and its shaded edge for the
/// arrows.
const RING: [u8; 4] = [191, 166, 122, 255];
const ARROW: [u8; 4] = [107, 82, 51, 230];

/// Paints the ring and the arrows for this turn.
pub(super) fn paint_ring(pixels: &mut [u8], size: u32, turn: u16) {
    let pixel = 1.0 / size as f32;
    let arrows = gaps(turn).map(Arrow::at);
    for y in 0..size {
        for x in 0..size {
            let at = (Vec2::new(x as f32, y as f32) + 0.5) * pixel;
            let from_middle = at.distance(MIDDLE);
            // Nothing is drawn well inside the ring.
            if from_middle < ARROW_RADIUS - HEAD_HALF * 2.0 {
                continue;
            }
            let index = ((y * size + x) * 4) as usize;
            let ring = (from_middle - RING_RADIUS).abs() - RING_HALF;
            blend(&mut pixels[index..index + 4], RING, cover(ring, pixel));
            let arrow = arrows
                .iter()
                .map(|arrow| arrow.distance(at, from_middle))
                .fold(f32::MAX, f32::min);
            blend(&mut pixels[index..index + 4], ARROW, cover(arrow, pixel));
        }
    }
}

/// One arrow: a curved shaft with a head at each end.
struct Arrow {
    /// The shaft's ends, in degrees clockwise from up.
    from: f32,
    to: f32,
    heads: [[Vec2; 3]; 2],
}

impl Arrow {
    fn at(gap: f32) -> Self {
        let (from, to) = (
            gap - ARROW_SPREAD + HEAD_LENGTH,
            gap + ARROW_SPREAD - HEAD_LENGTH,
        );
        let head = |base: f32, tip: f32| {
            [
                round_middle(tip, ARROW_RADIUS),
                round_middle(base, ARROW_RADIUS - HEAD_HALF),
                round_middle(base, ARROW_RADIUS + HEAD_HALF),
            ]
        };
        Self {
            from,
            to,
            heads: [head(from, gap - ARROW_SPREAD), head(to, gap + ARROW_SPREAD)],
        }
    }

    /// How far a point is outside it.
    fn distance(&self, at: Vec2, from_middle: f32) -> f32 {
        let angle = super::super::geometry::angle_round_middle(at);
        // The shaft: along the curve, between its ends.
        let along = |edge: f32| (angle - edge + 540.0).rem_euclid(360.0) - 180.0;
        let shaft = if along(self.from) >= 0.0 && along(self.to) <= 0.0 {
            (from_middle - ARROW_RADIUS).abs() - ARROW_HALF
        } else {
            f32::MAX
        };
        let heads = self
            .heads
            .iter()
            .map(|head| distance_to_shape(at, head))
            .fold(f32::MAX, f32::min);
        shaft.min(heads)
    }
}
