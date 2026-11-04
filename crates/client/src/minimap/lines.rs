//! Rivers and roads on the world map's maps, as lines over the land, only
//! where the land is known. Rivers come from the public seed, traced here a
//! few chunks at a time; roads are the ones the server has sent.

use std::collections::HashMap;

use bevy::platform::time::Instant;
use bevy::prelude::*;
use shared::poi::{CHUNK_SIZE, ChunkPos};
use shared::river::{MAX_LENGTH, River};
use shared::road::RoadKind;

use super::canvas::Canvas;
use super::projection::Projection;
use super::world::chunk_of;
use crate::common::CurrentWorld;

/// Rivers' blue, a little darker than the sea's on the maps so thin ones show.
const RIVER: [u8; 3] = [40, 90, 200];

/// How wide lines are, in pixels before the zoom: rivers from source to their
/// widest (a chunk's run each pixel more, as on the world map), and each kind
/// of road.
const RIVER_PIXELS: (f32, f32) = (1.0, 2.0);
fn road_pixels(kind: RoadKind) -> f32 {
    match kind {
        RoadKind::Path => 1.0,
        RoadKind::Track => 1.3,
        RoadKind::Road => 1.7,
        RoadKind::Highway => 2.4,
    }
}

/// A path is the land's own colour, darkened this much, so it reads on any
/// biome; tracks are dirt, roads stone, highways pale with dark edges.
const PATH_DARKER: f32 = 0.4;
const TRACK: [u8; 3] = [150, 100, 55];
const STONE: [u8; 3] = [130, 130, 130];
const HIGHWAY: [u8; 3] = [225, 215, 190];
const HIGHWAY_EDGE: [u8; 3] = [70, 65, 60];
/// How much of a highway's width its pale middle is.
const HIGHWAY_MIDDLE: f32 = 0.55;

/// Chunks each way a river's source can be from a point it reaches.
const SOURCE_REACH: i64 = MAX_LENGTH as i64 / CHUNK_SIZE + 1;

/// The rivers traced so far, by the chunk they start in (as the server numbers
/// chunks, wrapped round the planet).
#[derive(Resource, Default)]
pub struct MapRivers {
    rivers: HashMap<(i64, i64), Option<River>>,
}

impl MapRivers {
    /// Traces the rivers that could reach a box, nearest its middle first, for
    /// at most `budget_ms`. Returns whether any were left to do.
    pub fn fill(
        &mut self,
        world: &CurrentWorld,
        (min, max): ((f64, f64), (f64, f64)),
        budget_ms: f64,
    ) -> bool {
        let start = Instant::now();
        let width = world.surface.width();
        let columns = world.rivers.columns();
        let first = chunk_of(min.0.floor() as i64, min.1.floor() as i64, width);
        let across = ((max.0 - min.0) as i64 / CHUNK_SIZE + 1).min(columns);
        let rows = (max.1 - min.1) as i64 / CHUNK_SIZE + 1;
        let mut wanted = Vec::new();
        for dy in -SOURCE_REACH..=rows + SOURCE_REACH {
            for dx in -SOURCE_REACH..=across + SOURCE_REACH {
                let key = ((first.0 + dx).rem_euclid(columns), first.1 + dy);
                if !self.rivers.contains_key(&key) {
                    let (from_x, from_y) = (dx - across / 2, dy - rows / 2);
                    wanted.push((from_x * from_x + from_y * from_y, key));
                }
            }
        }
        wanted.sort();
        wanted.dedup_by_key(|(_, key)| *key);
        for (done, (_, (x, y))) in wanted.iter().enumerate() {
            if start.elapsed().as_secs_f64() * 1000.0 > budget_ms {
                return done < wanted.len();
            }
            let river = world.rivers.in_chunk(ChunkPos { x: *x, y: *y });
            self.rivers.insert((*x, *y), river);
        }
        false
    }

    /// Draws the rivers and roads over the land, only where it is known, the
    /// larger roads on top and roads over rivers (on bridges). `widen` scales
    /// the lines (the zoom and the screen's pixels); pixels further than `clip`
    /// from the middle are left alone.
    pub fn draw(
        &self,
        canvas: &mut Canvas,
        view: &Projection,
        world: &CurrentWorld,
        widen: f32,
        clip: f32,
    ) {
        let width = world.surface.width();
        let known = |(x, y): (i64, i64)| {
            let (column, row) = chunk_of(x, y, width);
            world.pois.is_known(ChunkPos { x: column, y: row })
        };
        let mut layer = Layer::new(canvas.width, canvas.height);
        for river in self.rivers.values().flatten() {
            let thickness = |i: usize| {
                let (thin, wide) = RIVER_PIXELS;
                (thin + i as f32 / CHUNK_SIZE as f32).min(wide) * widen
            };
            layer.stroke(view, &river.points, width, &known, thickness);
        }
        layer.paint(canvas, clip, RIVER, 1.0);
        // A kind at a time, smallest first, so larger roads are on top.
        for kind in RoadKind::ALL {
            let roads: Vec<_> = world.roads.iter().filter(|r| r.kind == kind).collect();
            if roads.is_empty() {
                continue;
            }
            let pixels = road_pixels(kind) * widen;
            let mut layer = Layer::new(canvas.width, canvas.height);
            for road in &roads {
                layer.stroke(view, &road.points, width, &known, |_| pixels);
            }
            match kind {
                RoadKind::Path => layer.paint(canvas, clip, [0, 0, 0], PATH_DARKER),
                RoadKind::Track => layer.paint(canvas, clip, TRACK, 1.0),
                RoadKind::Road => layer.paint(canvas, clip, STONE, 1.0),
                RoadKind::Highway => {
                    layer.paint(canvas, clip, HIGHWAY_EDGE, 1.0);
                    let mut middle = Layer::new(canvas.width, canvas.height);
                    let narrow = pixels * HIGHWAY_MIDDLE;
                    for road in &roads {
                        middle.stroke(view, &road.points, width, &known, |_| narrow);
                    }
                    middle.paint(canvas, clip, HIGHWAY, 1.0);
                }
            }
        }
    }
}

/// How much of each pixel lines cover, `0..=1`, so where a line's pieces
/// overlap it is painted once rather than darker.
struct Layer {
    width: u32,
    height: u32,
    cover: Vec<f32>,
}

impl Layer {
    fn new(width: u32, height: u32) -> Self {
        Self {
            width,
            height,
            cover: vec![0.0; (width * height) as usize],
        }
    }

    /// Adds a line through world grid points (one step apart, `x` not
    /// wrapped), `thickness(i)` pixels wide at point `i`, leaving out where
    /// `known` says the land isn't known.
    fn stroke(
        &mut self,
        view: &Projection,
        points: &[(i64, i64)],
        world_width: i64,
        known: &impl Fn((i64, i64)) -> bool,
        thickness: impl Fn(usize) -> f32,
    ) {
        let Some(&(start_x, _)) = points.first() else {
            return;
        };
        // The copy of the line nearest the middle of the map.
        let copies = ((view.centre.0 - start_x as f64) / world_width as f64).round() as i64;
        let shift = copies * world_width;
        let mut last: Option<(f32, f32)> = None;
        for (i, &(x, y)) in points.iter().enumerate() {
            let at = view.to_picture((x + shift) as f64, y as f64);
            let is_last = i + 1 == points.len();
            // Points closer than a pixel add nothing but time.
            if let Some(from) = last
                && (at.0 - from.0).hypot(at.1 - from.1) < 1.0
                && !is_last
            {
                continue;
            }
            if let Some(from) = last
                && known((x, y))
            {
                self.segment(from, at, thickness(i) / 2.0);
            }
            last = Some(at);
        }
    }

    /// Covers a stretch from `a` to `b`, `half` pixels each side, round-ended.
    fn segment(&mut self, a: (f32, f32), b: (f32, f32), half: f32) {
        let reach = half + 1.0;
        let low = |a: f32, b: f32| (a.min(b) - reach).floor().max(0.0) as u32;
        let high = |a: f32, b: f32, end: u32| ((a.max(b) + reach).ceil().max(0.0) as u32).min(end);
        let (along_x, along_y) = (b.0 - a.0, b.1 - a.1);
        let length = (along_x * along_x + along_y * along_y).max(1e-6);
        for y in low(a.1, b.1)..high(a.1, b.1, self.height) {
            for x in low(a.0, b.0)..high(a.0, b.0, self.width) {
                let (px, py) = (x as f32 + 0.5 - a.0, y as f32 + 0.5 - a.1);
                let t = ((px * along_x + py * along_y) / length).clamp(0.0, 1.0);
                let distance = (px - along_x * t).hypot(py - along_y * t) - half;
                let covered = (0.5 - distance).clamp(0.0, 1.0);
                let slot = &mut self.cover[(y * self.width + x) as usize];
                *slot = slot.max(covered);
            }
        }
    }

    /// Mixes `colour` over the canvas where covered, by `strength` at most.
    fn paint(&self, canvas: &mut Canvas, clip: f32, colour: [u8; 3], strength: f32) {
        let middle = (self.width as f32 / 2.0, self.height as f32 / 2.0);
        for (index, &covered) in self.cover.iter().enumerate() {
            if covered <= 0.0 {
                continue;
            }
            let (x, y) = (index as u32 % self.width, index as u32 / self.width);
            let (dx, dy) = (x as f32 + 0.5 - middle.0, y as f32 + 0.5 - middle.1);
            if dx.hypot(dy) <= clip {
                canvas.blend(x, y, colour, covered * strength);
            }
        }
    }
}

#[cfg(test)]
mod tests;
