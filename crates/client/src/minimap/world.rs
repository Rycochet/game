//! The world map's map: the land's plain colours, seen from above, in the
//! chunks the server has sent. Colours are kept per chunk, worked out a few at
//! a time, since the full map can show hundreds.

use std::collections::HashMap;

use bevy::platform::time::Instant;
use bevy::prelude::*;
use shared::movement::WorldConditions;
use shared::poi::{CHUNK_SIZE, ChunkPos};

use super::canvas::Canvas;
use super::projection::Projection;
use crate::common::CurrentWorld;
use crate::surface_style::SurfaceStyle;

/// World points per stored colour: one each, for the full map zoomed in. Where
/// a pixel covers more than that (the corner minimap), `draw` averages them.
const STEP: i64 = 1;
const SIDE: usize = (CHUNK_SIZE / STEP) as usize;

/// A chunk's colours, and the water and snow they were worked out for.
struct Tile {
    made_for: (u64, u64),
    /// Colours across a row: the last column of chunks is wider than the rest.
    across: usize,
    colours: Vec<[u8; 3]>,
}

/// The colours worked out so far, by chunk (as the server numbers them, so
/// wrapped round the planet).
#[derive(Resource, Default)]
pub struct WorldTiles {
    tiles: HashMap<(i64, i64), Tile>,
}

/// How many columns of chunks go round the planet. As `shared::poi`
/// numbers them: what is left over past the last whole chunk belongs to it.
fn columns(width: i64) -> i64 {
    (width / CHUNK_SIZE).max(1)
}

/// Which chunk a world position is in, wrapping round the planet.
fn chunk_of(x: i64, y: i64, width: i64) -> (i64, i64) {
    (
        (x.rem_euclid(width) / CHUNK_SIZE).min(columns(width) - 1),
        y.div_euclid(CHUNK_SIZE),
    )
}

/// The columns of chunks a span of `x` crosses, each once.
fn columns_between(min_x: i64, max_x: i64, width: i64) -> Vec<i64> {
    if max_x - min_x >= width {
        return (0..columns(width)).collect();
    }
    let last = chunk_of(max_x, 0, width).0;
    let mut column = chunk_of(min_x, 0, width).0;
    let mut found = vec![column];
    while column != last {
        column = (column + 1) % columns(width);
        found.push(column);
    }
    found
}

impl WorldTiles {
    /// Works out colours for known chunks in a box that lack them or have out
    /// of date ones, nearest to its centre first, for at most `budget_ms`.
    /// Returns whether any were left to do.
    pub fn fill(
        &mut self,
        world: &CurrentWorld,
        style: &SurfaceStyle,
        conditions: WorldConditions,
        (min, max): ((f64, f64), (f64, f64)),
        budget_ms: f64,
    ) -> bool {
        let start = Instant::now();
        let width = world.surface.width();
        let made_for = (conditions.water.to_bits(), conditions.snow_line.to_bits());
        let row = |at: f64| (at.floor() as i64).div_euclid(CHUNK_SIZE);
        let columns = columns_between(min.0.floor() as i64, max.0.ceil() as i64, width);
        let middle = ((min.0 + max.0) / 2.0, (min.1 + max.1) / 2.0);
        let mut wanted = Vec::new();
        for y in row(min.1)..=row(max.1) {
            for &x in &columns {
                let key = (x, y);
                let up_to_date = self.tiles.get(&key).is_some_and(|t| t.made_for == made_for);
                if !up_to_date && world.pois.is_known(ChunkPos { x, y }) {
                    // The shorter way round the planet.
                    let half = (width / 2) as f64;
                    let east = (x * CHUNK_SIZE) as f64 - middle.0;
                    let east = (east + half).rem_euclid(width as f64) - half;
                    let south = (y * CHUNK_SIZE) as f64 - middle.1;
                    wanted.push((east.hypot(south), key));
                }
            }
        }
        wanted.sort_by(|a, b| a.0.total_cmp(&b.0));
        for (done, (_, key)) in wanted.iter().enumerate() {
            if start.elapsed().as_secs_f64() * 1000.0 > budget_ms {
                return done < wanted.len();
            }
            let (across, colours) = tile_colours(world, style, conditions, *key);
            self.tiles.insert(
                *key,
                Tile {
                    made_for,
                    across,
                    colours,
                },
            );
        }
        false
    }

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
        // Neighbouring pixels are mostly in the same chunk.
        let mut last: Option<((i64, i64), Option<&Tile>)> = None;
        for y in 0..canvas.height {
            for x in 0..canvas.width {
                let (px, py) = (x as f32 + 0.5, y as f32 + 0.5);
                if (px - view.middle.0).hypot(py - view.middle.1) > clip {
                    continue;
                }
                let mut sum = [0u32; 3];
                for (offset_x, offset_y) in samples {
                    let (grid_x, grid_y) = view.to_grid(x as f32 + offset_x, y as f32 + offset_y);
                    // Wrapped first: chunks line up from the seam at `x` = 0.
                    let grid_x = (grid_x.floor() as i64).rem_euclid(width);
                    let grid_y = grid_y.floor() as i64;
                    let key = chunk_of(grid_x, grid_y, width);
                    let tile = match last {
                        Some((last_key, tile)) if last_key == key => tile,
                        _ => {
                            let tile = self.tiles.get(&key);
                            last = Some((key, tile));
                            tile
                        }
                    };
                    let colour = tile.map_or(parchment, |tile| {
                        let column = (grid_x - key.0 * CHUNK_SIZE) / STEP;
                        let row = grid_y.rem_euclid(CHUNK_SIZE) / STEP;
                        tile.colours[row as usize * tile.across + column as usize]
                    });
                    for (total, channel) in sum.iter_mut().zip(colour) {
                        *total += u32::from(channel);
                    }
                }
                canvas.set(x, y, sum.map(|total| (total / samples.len() as u32) as u8));
            }
        }
    }
}

/// A chunk's colours, and how many there are across a row.
fn tile_colours(
    world: &CurrentWorld,
    style: &SurfaceStyle,
    conditions: WorldConditions,
    (chunk_x, chunk_y): (i64, i64),
) -> (usize, Vec<[u8; 3]>) {
    let width = world.surface.width();
    let wide = if chunk_x == columns(width) - 1 {
        width - chunk_x * CHUNK_SIZE
    } else {
        CHUNK_SIZE
    };
    let across = (wide + STEP - 1) / STEP;
    let mut colours = Vec::with_capacity(across as usize * SIDE);
    for row in 0..SIDE as i64 {
        for column in 0..across {
            let x = chunk_x * CHUNK_SIZE + column * STEP;
            let y = chunk_y * CHUNK_SIZE + row * STEP;
            let sample = world.surface.sample(x, y);
            let climate = world.climate.sample(x, y, sample.height);
            let [red, green, blue, _] =
                style.ground_colour(sample, climate, conditions.water, conditions.snow_line);
            colours.push([red, green, blue]);
        }
    }
    (across as usize, colours)
}

/// Whether any known chunk overlaps a box.
pub fn shows_known(world: &CurrentWorld, (min, max): ((f64, f64), (f64, f64))) -> bool {
    let row = |at: f64| (at.floor() as i64).div_euclid(CHUNK_SIZE);
    let columns = columns_between(
        min.0.floor() as i64,
        max.0.ceil() as i64,
        world.surface.width(),
    );
    (row(min.1)..=row(max.1)).any(|y| {
        columns
            .iter()
            .any(|&x| world.pois.is_known(ChunkPos { x, y }))
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_column_is_found_with_the_last_one_wider() {
        // 650 wide: ten columns, the last of them 74 wide.
        assert_eq!(columns_between(0, 100, 650), [0, 1]);
        assert_eq!(columns_between(-20, 5, 650), [9, 0]);
        assert_eq!(columns_between(-100, -20, 650), [8, 9]);
        assert_eq!(columns_between(-5, 700, 650).len(), 10);
        assert_eq!(chunk_of(649, 0, 650), (9, 0));
    }

    #[test]
    fn chunks_wrap_round_the_planet_only_east_west() {
        assert_eq!(chunk_of(0, 0, 640), (0, 0));
        assert_eq!(chunk_of(-1, -1, 640), (9, -1));
        assert_eq!(chunk_of(640, 64, 640), (0, 1));
    }
}
