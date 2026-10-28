//! Draws the world map: the surface around a point, seen from the south as a
//! stack of height slices, each grid column drawn as a vertical line.
//!
//! Faithful port of the TypeScript prototype's `WorldMap.tsx`. Slices are
//! drawn front (south) to back; each only paints the part of its column that
//! sticks up above the previous slice.

use shared::surface::Surface;

use crate::surface_style::{SurfaceStyle, hsl_to_rgba};

/// Image width in pixels (one grid column per pixel at the front).
pub const WIDTH: usize = 400;
/// How many grid rows deep the view is (every second row is drawn).
pub const DEPTH: i64 = 400;
/// Empty space above the horizon for tall terrain.
pub const SKY: i64 = 64;
/// Image height in pixels.
pub const HEIGHT: usize = (DEPTH / 2 + SKY) as usize;

/// Renders into `pixels` (RGBA, `WIDTH * HEIGHT * 4` bytes), centred on the
/// grid position `(center_x, center_y)`.
pub fn render(
    surface: &Surface,
    style: &SurfaceStyle,
    water_level: f64,
    snow_line: f64,
    center_x: i64,
    center_y: i64,
    pixels: &mut [u8],
) {
    assert_eq!(pixels.len(), WIDTH * HEIGHT * 4);
    for pixel in pixels.as_chunks_mut::<4>().0 {
        *pixel = [0, 0, 0, 255];
    }

    let width = WIDTH as f64;
    for column in 0..WIDTH {
        let screen_x = column as f64;
        // Screen row of the top of the previous slice. Starts below the image.
        let mut top = HEIGHT as i64;
        let mut first = true;

        for row in (0..=DEPTH).rev().step_by(2) {
            // Perspective: further slices are narrower on screen...
            let shift = (DEPTH - row) as f64 / 6.0;
            if screen_x < shift || screen_x > width - shift {
                continue;
            }
            // ...and each pixel covers more of the world.
            let spread = (4.0 * shift + width) / width;
            let x = center_x + (spread * (screen_x - width / 2.0)).floor() as i64;
            let y = center_y + row - DEPTH / 2;

            let sample = surface.sample(x, y);
            // Slope towards the west, for shading.
            let west = surface.sample(x - 1, y).height;
            let (band, lightness) = style.shade(sample, west, water_level, snow_line, 1.0);

            let base = row / 2;
            let bottom = top;
            top = (base as f64 - sample.height.max(water_level) / 2.0).ceil() as i64;

            if first {
                // Front slice: a cut-away showing the water depth, then earth
                // down to the base line.
                first = false;
                let ground = (base as f64 - sample.height / 2.0).ceil() as i64;
                if top != ground {
                    let colour = hsl_to_rgba(band.hue, band.saturation, lightness / 2.0);
                    fill_column(pixels, column, top, ground, colour);
                }
                let (h, s, l) = style.earth_face;
                fill_column(pixels, column, ground, base, hsl_to_rgba(h, s, l));
            } else if bottom > top {
                let colour = hsl_to_rgba(band.hue, band.saturation, lightness);
                fill_column(pixels, column, top, bottom, colour);
            }
        }
    }
}

/// Paints rows `from..to` (in map space, before the sky offset) of a column.
fn fill_column(pixels: &mut [u8], column: usize, from: i64, to: i64, colour: [u8; 4]) {
    let from = (from + SKY).clamp(0, HEIGHT as i64) as usize;
    let to = (to + SKY).clamp(0, HEIGHT as i64) as usize;
    for row in from..to {
        let offset = (row * WIDTH + column) * 4;
        pixels[offset..offset + 4].copy_from_slice(&colour);
    }
}
