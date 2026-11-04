//! Painting the kept colours of the world map's chunks onto a map.

use shared::poi::CHUNK_SIZE;

use super::{Tile, WorldTiles, chunk_of, columns};
use crate::minimap::canvas::Canvas;
use crate::minimap::projection::Projection;

impl WorldTiles {
    /// Paints the chunks it has, unknown ones staying `parchment`. Pixels
    /// further than `clip` from the middle of the picture are left alone.
    pub fn draw(
        &self,
        canvas: &mut Canvas,
        view: &Projection,
        width: i64,
        clip: f32,
        parchment: [u8; 3],
    ) {
        let samples: &[(f32, f32)] = if view.units_per_pixel > 1.0 {
            &[(0.25, 0.25), (0.75, 0.25), (0.25, 0.75), (0.75, 0.75)]
        } else {
            &[(0.5, 0.5)]
        };
        // How far the grid moves for a pixel across and down, so each sample's
        // place is added to rather than worked out afresh.
        let origin = view.to_grid(0.0, 0.0);
        let right = view.to_grid(1.0, 0.0);
        let down = view.to_grid(0.0, 1.0);
        let across = (right.0 - origin.0, right.1 - origin.1);
        let along = (down.0 - origin.0, down.1 - origin.1);
        let mut lookup = Lookup {
            tiles: self,
            width,
            last: None,
        };
        let clip_squared = clip * clip;
        for y in 0..canvas.height {
            // The pixels of this row whose middles are within `clip`.
            let off_middle = y as f32 + 0.5 - view.middle.1;
            let left = clip_squared - off_middle * off_middle;
            if left < 0.0 {
                continue;
            }
            let half = left.sqrt();
            let first = (view.middle.0 - half - 0.5).ceil().max(0.0) as u32;
            let end = ((view.middle.0 + half - 0.5).floor() + 1.0).clamp(0.0, canvas.width as f32);
            let start = (y * canvas.width * 4) as usize;
            let row = &mut canvas.pixels[start..start + canvas.width as usize * 4];
            for x in first..end as u32 {
                let at = x as usize * 4;
                let colour_at = |lookup: &mut Lookup, (offset_x, offset_y): (f32, f32)| {
                    let (px, py) = (
                        f64::from(x as f32 + offset_x),
                        f64::from(y as f32 + offset_y),
                    );
                    let grid_x = origin.0 + px * across.0 + py * along.0;
                    let grid_y = origin.1 + px * across.1 + py * along.1;
                    lookup
                        .colour(floor(grid_x), floor(grid_y))
                        .unwrap_or(parchment)
                };
                let [red, green, blue] = if let [only] = samples {
                    colour_at(&mut lookup, *only)
                } else {
                    let mut sum = [0u32; 3];
                    for &offset in samples {
                        let colour = colour_at(&mut lookup, offset);
                        for channel in 0..3 {
                            sum[channel] += u32::from(colour[channel]);
                        }
                    }
                    sum.map(|total| (total / samples.len() as u32) as u8)
                };
                row[at..at + 4].copy_from_slice(&[red, green, blue, 255]);
            }
        }
    }
}

/// The whole number at or below `value`: `f64::floor` is a call into the
/// maths library on most PCs, and this runs for every pixel.
fn floor(value: f64) -> i64 {
    let truncated = value as i64;
    if (truncated as f64) > value {
        truncated - 1
    } else {
        truncated
    }
}

/// Finds the colour at grid points, remembering the last chunk's bounds:
/// neighbouring pixels are mostly in the same chunk, and checking the bounds
/// is far quicker than working out which chunk a point is in.
struct Lookup<'a> {
    tiles: &'a WorldTiles,
    width: i64,
    /// The last chunk: its west and north edges, how wide it is, and its tile.
    last: Option<(i64, i64, i64, Option<&'a Tile>)>,
}

impl<'a> Lookup<'a> {
    /// The colour kept for a grid point, if its chunk has been worked out.
    fn colour(&mut self, x: i64, y: i64) -> Option<[u8; 3]> {
        // Wrapped first: chunks line up from the seam at `x` = 0.
        let x = if (0..self.width).contains(&x) {
            x
        } else {
            x.rem_euclid(self.width)
        };
        let (west, north, _, tile) = match self.last {
            Some(last @ (west, north, wide, _))
                if (west..west + wide).contains(&x) && (north..north + CHUNK_SIZE).contains(&y) =>
            {
                last
            }
            _ => {
                let key = chunk_of(x, y, self.width);
                let west = key.0 * CHUNK_SIZE;
                let wide = if key.0 == columns(self.width) - 1 {
                    self.width - west
                } else {
                    CHUNK_SIZE
                };
                let last = (west, key.1 * CHUNK_SIZE, wide, self.tiles.tiles.get(&key));
                self.last = Some(last);
                last
            }
        };
        let tile = tile?;
        let (column, row) = (x - west, y - north);
        let at = if tile.step == 1 {
            row * tile.across as i64 + column
        } else {
            row / tile.step * tile.across as i64 + column / tile.step
        };
        Some(tile.colours[at as usize])
    }
}

#[cfg(test)]
mod tests;
