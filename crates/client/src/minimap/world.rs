//! The world map's map: the land's plain colours, seen from above, in the
//! chunks the server has sent. Colours are kept per chunk, worked out a few at
//! a time, since the full map can show hundreds: first a quick, coarse version
//! of every chunk in view, then full detail where the zoom shows it.

use std::collections::HashMap;

use bevy::platform::time::Instant;
use bevy::prelude::*;
use shared::movement::WorldConditions;
use shared::poi::{CHUNK_SIZE, ChunkPos};

use crate::common::{CurrentWorld, NORMAL_SNOW_LINE};
use crate::surface_style::SurfaceStyle;

/// World points per stored colour in a chunk's first, quick version: a
/// sixteenth of the work, so the whole map fills at once and sharpens after.
/// Pixels covering at least this many points never need more.
const COARSE_STEP: i64 = 4;

/// A chunk's colours, one per `step` world points each way.
struct Tile {
    /// `COARSE_STEP`, or 1 once full detail is worked out.
    step: i64,
    /// Colours across a row: the last column of chunks is wider than the rest.
    across: usize,
    colours: Vec<[u8; 3]>,
}

/// The colours worked out so far, by chunk (as the server numbers them, so
/// wrapped round the planet).
#[derive(Resource, Default)]
pub struct WorldTiles {
    tiles: HashMap<(i64, i64), Tile>,
    /// When the work still to do began, for the log line when it is done.
    working_since: Option<Instant>,
}

/// What the maps show (owner's decision): the world as it normally is, water
/// at mid-tide and snow at its normal line, whatever the tide or weather, so
/// a chunk's colours never go out of date.
fn map_conditions(world: &CurrentWorld) -> WorldConditions {
    WorldConditions {
        water: world.params.sea_level,
        snow_line: NORMAL_SNOW_LINE,
    }
}

/// The step a chunk should be worked out at next, if any: a coarse version
/// first, then full detail only where a pixel covers fewer points than that.
fn next_step(tile: Option<&Tile>, units_per_pixel: f64) -> Option<i64> {
    match tile {
        None => Some(COARSE_STEP),
        Some(tile) if tile.step > 1 && units_per_pixel < COARSE_STEP as f64 => Some(1),
        Some(_) => None,
    }
}

/// How many columns of chunks go round the planet. As `shared::poi`
/// numbers them: what is left over past the last whole chunk belongs to it.
fn columns(width: i64) -> i64 {
    (width / CHUNK_SIZE).max(1)
}

/// Which chunk a world position is in, wrapping round the planet.
pub(super) fn chunk_of(x: i64, y: i64, width: i64) -> (i64, i64) {
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
    /// Forgets every chunk's colours, as when the colours they are drawn in
    /// change.
    pub fn clear(&mut self) {
        self.tiles.clear();
    }

    /// Works out colours for known chunks in a box: first a coarse version of
    /// every one, then full detail where `units_per_pixel` (world points to a
    /// pixel) shows it, nearest to the box's centre first, for at most
    /// `budget_ms`. Returns whether any were left to do.
    pub fn fill(
        &mut self,
        world: &CurrentWorld,
        style: &SurfaceStyle,
        (min, max): ((f64, f64), (f64, f64)),
        units_per_pixel: f64,
        budget_ms: f64,
    ) -> bool {
        let start = Instant::now();
        let width = world.surface.width();
        let row = |at: f64| (at.floor() as i64).div_euclid(CHUNK_SIZE);
        let columns = columns_between(min.0.floor() as i64, max.0.ceil() as i64, width);
        let middle = ((min.0 + max.0) / 2.0, (min.1 + max.1) / 2.0);
        let mut wanted = Vec::new();
        for y in row(min.1)..=row(max.1) {
            for &x in &columns {
                let key = (x, y);
                let Some(step) = next_step(self.tiles.get(&key), units_per_pixel) else {
                    continue;
                };
                if world.pois.is_known(ChunkPos { x, y }) {
                    // The shorter way round the planet.
                    let half = (width / 2) as f64;
                    let east = (x * CHUNK_SIZE) as f64 - middle.0;
                    let east = (east + half).rem_euclid(width as f64) - half;
                    let south = (y * CHUNK_SIZE) as f64 - middle.1;
                    // Coarse ones first: the larger step.
                    wanted.push((-step, east.hypot(south), key, step));
                }
            }
        }
        if wanted.is_empty() {
            if let Some(since) = self.working_since.take() {
                debug!("Map tiles all worked out in {:.2?}", since.elapsed());
            }
            return false;
        }
        self.working_since.get_or_insert(start);
        wanted.sort_by(|a, b| a.0.cmp(&b.0).then(a.1.total_cmp(&b.1)));
        let conditions = map_conditions(world);
        for (done, &(_, _, key, step)) in wanted.iter().enumerate() {
            if start.elapsed().as_secs_f64() * 1000.0 > budget_ms {
                return done < wanted.len();
            }
            let tile = tile_colours(world, style, conditions, key, step);
            self.tiles.insert(key, tile);
        }
        false
    }
}

/// A chunk's colours, one for every `step` points each way, from the middle
/// of the square each stands for.
fn tile_colours(
    world: &CurrentWorld,
    style: &SurfaceStyle,
    conditions: WorldConditions,
    (chunk_x, chunk_y): (i64, i64),
    step: i64,
) -> Tile {
    let width = world.surface.width();
    let wide = if chunk_x == columns(width) - 1 {
        width - chunk_x * CHUNK_SIZE
    } else {
        CHUNK_SIZE
    };
    let across = (wide + step - 1) / step;
    let rows = CHUNK_SIZE / step;
    let mut colours = Vec::with_capacity((across * rows) as usize);
    for row in 0..rows {
        for column in 0..across {
            let x = chunk_x * CHUNK_SIZE + (column * step + step / 2).min(wide - 1);
            let y = chunk_y * CHUNK_SIZE + row * step + step / 2;
            let sample = world.surface.sample(x, y);
            let climate = world.climate.sample(x, y, sample.height);
            let [red, green, blue, _] =
                style.ground_colour(sample, climate, conditions.water, conditions.snow_line);
            colours.push([red, green, blue]);
        }
    }
    Tile {
        step,
        across: across as usize,
        colours,
    }
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

mod paint;

#[cfg(test)]
mod tests;
