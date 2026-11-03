//! Painting the level's cells.

use super::super::canvas::Canvas;
use super::super::projection::Projection;
use super::{Looks, colour_at};

/// Paints the level. Pixels further than `clip` from the middle of the picture
/// are left alone.
pub fn draw(canvas: &mut Canvas, view: &Projection, looks: &Looks, parchment: [u8; 3], clip: f32) {
    let line = 0.5 * view.units_per_pixel;
    // When turned, a few samples per pixel keep the lines from breaking up.
    let samples: &[(f32, f32)] = if view.is_square_on() {
        &[(0.5, 0.5)]
    } else {
        &[(0.25, 0.25), (0.75, 0.25), (0.25, 0.75), (0.75, 0.75)]
    };
    let colour_of = |x: f32, y: f32| {
        let (grid_x, grid_y) = view.to_grid(x, y);
        let (cell_x, cell_y) = ((grid_x + 0.5).floor(), (grid_y + 0.5).floor());
        let look = looks.at(cell_x as i64, cell_y as i64);
        colour_at(look, grid_x - cell_x, grid_y - cell_y, line).unwrap_or(parchment)
    };
    for y in 0..canvas.height {
        for x in 0..canvas.width {
            let (px, py) = (x as f32 + 0.5, y as f32 + 0.5);
            if (px - view.middle.0).hypot(py - view.middle.1) > clip {
                continue;
            }
            let mut sum = [0u32; 3];
            for (offset_x, offset_y) in samples {
                let colour = colour_of(x as f32 + offset_x, y as f32 + offset_y);
                for (total, channel) in sum.iter_mut().zip(colour) {
                    *total += u32::from(channel);
                }
            }
            canvas.set(x, y, sum.map(|total| (total / samples.len() as u32) as u8));
        }
    }
}
