//! Rivers near the player, kept ready to draw: traced once from their chunk,
//! then smoothed, with a width at every point. `terrain::build_rivers` turns
//! them into the mesh.

use std::collections::BTreeMap;

use bevy::prelude::*;
use shared::climate::Climate;
use shared::feature::{CHUNK_SIZE, ChunkPos};
use shared::river::{MAX_LENGTH, River, Rivers};
use shared::surface::Surface;

/// World map grid units per screen pixel in the starting view (a 400-point
/// window is about 1250 pixels wide), so widths can be given in pixels.
const UNITS_PER_PIXEL: f32 = 0.32;

/// Width at the source and the widest a river gets, in pixels at the starting
/// zoom (the owner's figures). Width comes only from how far it has run.
const SOURCE_PIXELS: f32 = 1.0;
const WIDEST_PIXELS: f32 = 4.0;

/// Points a river runs for each extra pixel of width: a chunk (the owner's
/// figure), so short rivers stay thin wherever they end.
const POINTS_PER_PIXEL: f32 = CHUNK_SIZE as f32;

/// Points either side averaged to take the grid's corners out of a course.
const SMOOTHING: usize = 4;

/// Chunks each way from the player a source can be and still reach the window.
const CHUNK_REACH: i64 = (MAX_LENGTH as i64 + super::terrain::REACH) / CHUNK_SIZE + 1;

/// A river ready to draw.
pub struct DrawnRiver {
    /// Where it starts, as a world grid position.
    pub source: (i64, i64),
    /// Smoothed course, relative to the source.
    pub points: Vec<Vec2>,
    /// Width at each point, in grid units.
    pub widths: Vec<f32>,
    /// Temperature at each point, which decides if it is frozen.
    pub temperatures: Vec<f64>,
}

impl DrawnRiver {
    fn new(river: &River, climate: &Climate, surface: &Surface) -> Self {
        let source = river.points[0];
        let last = river.points.len() - 1;
        let relative = |i: usize| {
            let (x, y) = river.points[i];
            Vec2::new((x - source.0) as f32, (y - source.1) as f32)
        };
        let points = (0..=last)
            .map(|i| {
                // A shorter window near the ends, so they stay where they are.
                let reach = SMOOTHING.min(i).min(last - i);
                let total: Vec2 = (i - reach..=i + reach).map(relative).sum();
                total / (2 * reach + 1) as f32
            })
            .collect();
        let widths = (0..=last)
            .map(|i| {
                let extra = i as f32 / POINTS_PER_PIXEL;
                (SOURCE_PIXELS + extra).min(WIDEST_PIXELS) * UNITS_PER_PIXEL
            })
            .collect();
        let temperatures = river
            .points
            .iter()
            .map(|&(x, y)| {
                climate
                    .sample(x, y, surface.sample(x, y).height)
                    .temperature
            })
            .collect();
        Self {
            source,
            points,
            widths,
            temperatures,
        }
    }
}

/// The chunks around the player and their rivers. A chunk stays here until the
/// player has gone well away, since tracing is the slow part.
#[derive(Resource, Default)]
pub struct KnownRivers {
    chunks: BTreeMap<ChunkPos, Option<DrawnRiver>>,
    /// The chunk last made sure of, so most steps do nothing.
    around: Option<ChunkPos>,
}

impl KnownRivers {
    /// Makes sure every chunk whose river could reach the window is known.
    pub fn update(
        &mut self,
        rivers: &Rivers,
        climate: &Climate,
        surface: &Surface,
        (x, y): (i64, i64),
    ) {
        let columns = rivers.columns();
        let here = ChunkPos {
            x: x.rem_euclid(rivers.width()).div_euclid(CHUNK_SIZE),
            y: y.div_euclid(CHUNK_SIZE),
        };
        if self.around == Some(here) {
            return;
        }
        self.around = Some(here);
        for dy in -CHUNK_REACH..=CHUNK_REACH {
            for dx in -CHUNK_REACH..=CHUNK_REACH {
                let chunk = ChunkPos {
                    x: (here.x + dx).rem_euclid(columns),
                    y: here.y + dy,
                };
                self.chunks.entry(chunk).or_insert_with(|| {
                    let river = rivers.in_chunk(chunk);
                    river.map(|river| DrawnRiver::new(&river, climate, surface))
                });
            }
        }
        // Forget chunks a good way beyond that (east-west, measured round the
        // planet).
        let keep = CHUNK_REACH + 4;
        self.chunks.retain(|chunk, _| {
            let east = (chunk.x - here.x).rem_euclid(columns);
            east.min(columns - east) <= keep && (chunk.y - here.y).abs() <= keep
        });
    }

    pub fn rivers(&self) -> impl Iterator<Item = &DrawnRiver> {
        self.chunks.values().flatten()
    }
}
