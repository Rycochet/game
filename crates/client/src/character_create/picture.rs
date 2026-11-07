//! Drawing the chooser: the ring they turn on, with arrows in the gaps, then each
//! triangle with its radar chart behind it, the kind's bonus as its inner layer,
//! and its dot. Painted pixel by pixel into an image,
//! again whenever the draft changes.

use bevy::math::Vec2;
use shared::sheet::{Attributes, Build, Row};

mod ring;

use super::geometry::{chart, corners, distance_to_line, distance_to_shape, dot_place};

/// How many pixels across the picture is.
pub(super) const PIXELS: u32 = 640;

/// The bonus layer's round ends, as a share of the picture.
const BONUS_ROUND: f32 = 0.012;
const DOT_RADIUS: f32 = 0.02;
/// Half the dot's ring's width.
const DOT_RING: f32 = 0.003;

const TRIANGLE: [u8; 4] = [250, 242, 220, 255];
const INK: [u8; 4] = [64, 43, 26, 255];
const AXIS: [u8; 4] = [64, 43, 26, 70];
const BONUS: [u8; 4] = [200, 150, 40, 235];
const DOT_LIGHT: [u8; 4] = [250, 242, 220, 255];

/// Each row's chart colour: what its triangle adds.
const fn row_colour(row: Row) -> [u8; 4] {
    let [red, green, blue] = crate::widgets::row_rgb(row);
    [red, green, blue, 150]
}

/// What is drawn: the build, the attributes it gives and the kind's bonus.
pub(super) struct Drawn<'a> {
    pub build: &'a Build,
    pub attributes: &'a Attributes,
    pub bonus: &'a Attributes,
    /// The row the keys move the dot of, drawn with a thicker edge.
    pub chosen: Row,
}

/// The picture's pixels, RGBA, `PIXELS` square.
pub(super) fn paint(drawn: &Drawn) -> Vec<u8> {
    let mut pixels = vec![0; (PIXELS * PIXELS * 4) as usize];
    ring::paint_ring(&mut pixels, PIXELS, drawn.build.turn);
    let pixel = 1.0 / PIXELS as f32;
    for row in Row::ALL {
        let shapes = Shapes::of(drawn, row);
        // Only the square round this triangle can be touched.
        let low = shapes.corners.iter().fold(Vec2::ONE, |low, c| low.min(*c)) - 0.03;
        let high = shapes
            .corners
            .iter()
            .fold(Vec2::ZERO, |high, c| high.max(*c))
            + 0.03;
        let from = (low * PIXELS as f32).max(Vec2::ZERO).as_uvec2();
        let to = (high * PIXELS as f32)
            .min(Vec2::splat(PIXELS as f32))
            .as_uvec2();
        for y in from.y..to.y {
            for x in from.x..to.x {
                let at = (Vec2::new(x as f32, y as f32) + 0.5) * pixel;
                let index = ((y * PIXELS + x) * 4) as usize;
                for (colour, coverage) in shapes.layers_at(at, pixel, drawn.chosen == row) {
                    blend(&mut pixels[index..index + 4], colour, coverage);
                }
            }
        }
    }
    pixels
}

/// One triangle's shapes.
struct Shapes {
    centre: Vec2,
    corners: [Vec2; 3],
    values: [Vec2; 3],
    bonus: [Vec2; 3],
    dot: Vec2,
    colour: [u8; 4],
}

impl Shapes {
    fn of(drawn: &Drawn, row: Row) -> Self {
        let turn = drawn.build.turn;
        let of = |attributes: &Attributes| row.attributes().map(|which| attributes.get(which));
        Self {
            centre: super::geometry::centre(row, turn),
            corners: corners(row, turn),
            values: chart(row, turn, of(drawn.attributes)),
            bonus: chart(row, turn, of(drawn.bonus)),
            dot: dot_place(row, drawn.build),
            colour: row_colour(row),
        }
    }

    /// What covers this point, back to front, and how much of it.
    fn layers_at(&self, at: Vec2, pixel: f32, chosen: bool) -> Vec<([u8; 4], f32)> {
        let row_shade = |distance: f32| cover(distance, pixel);
        let mut layers = vec![(TRIANGLE, row_shade(distance_to_shape(at, &self.corners)))];
        for corner in self.corners {
            layers.push((
                AXIS,
                cover(
                    distance_to_line(at, self.centre, corner) - pixel * 0.5,
                    pixel,
                ),
            ));
        }
        layers.push((self.colour, row_shade(distance_to_shape(at, &self.values))));
        // Round-ended, so a bonus of nothing still shows where it starts.
        let bonus = distance_to_shape(at, &self.bonus) - BONUS_ROUND;
        layers.push((BONUS, row_shade(bonus)));
        let edge = if chosen { pixel * 2.5 } else { pixel * 1.2 };
        let outline = distance_to_shape(at, &self.corners).abs() - edge;
        layers.push((INK, cover(outline, pixel)));
        // A ring, so the bonus shows through it at the middle.
        let ring = (at.distance(self.dot) - DOT_RADIUS).abs() - DOT_RING;
        layers.push((DOT_LIGHT, cover(ring - pixel, pixel)));
        layers.push((INK, cover(ring, pixel)));
        layers
    }
}

/// How much of a pixel `pixel` across a shape covers, from how far outside it the
/// pixel's middle is: smooth over one pixel, so edges aren't jagged.
fn cover(distance: f32, pixel: f32) -> f32 {
    (0.5 - distance / pixel).clamp(0.0, 1.0)
}

/// Paints `colour` over a pixel, `coverage` of it.
fn blend(pixel: &mut [u8], colour: [u8; 4], coverage: f32) {
    let alpha = colour[3] as f32 / 255.0 * coverage;
    if alpha <= 0.0 {
        return;
    }
    let under = pixel[3] as f32 / 255.0;
    let out = alpha + under * (1.0 - alpha);
    for channel in 0..3 {
        let mixed = colour[channel] as f32 * alpha + pixel[channel] as f32 * under * (1.0 - alpha);
        pixel[channel] = (mixed / out.max(f32::EPSILON)).round() as u8;
    }
    pixel[3] = (out * 255.0).round() as u8;
}

#[cfg(test)]
mod tests;
