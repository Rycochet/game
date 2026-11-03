//! The chunks and pois in an area.

use std::collections::BTreeSet;

use super::{ChunkPos, Poi, PoiMap};

impl PoiMap {
    /// Every chunk with any point within `min..=max` (world grid points, in
    /// any planet-width frame), each once, sorted.
    pub fn chunks_in_area(&self, min: (i64, i64), max: (i64, i64)) -> Vec<ChunkPos> {
        let mut chunks = BTreeSet::new();
        // An area wider than the planet covers every column.
        let (west, east) = if max.0 - min.0 >= self.width {
            (0, self.width - 1)
        } else {
            (min.0, max.0)
        };
        let (first, last) = (self.chunk_at(west, min.1), self.chunk_at(east, max.1));
        for cy in first.y..=last.y {
            // Across the seam at x = 0, the columns wrap round.
            let mut cx = first.x;
            loop {
                chunks.insert(ChunkPos { x: cx, y: cy });
                if cx == last.x {
                    break;
                }
                cx = (cx + 1) % self.columns();
            }
        }
        chunks.into_iter().collect()
    }

    /// Every poi whose position is within `min..=max` (world grid
    /// points, in any planet-width frame: the pois are given in the same
    /// one), sorted by position.
    pub fn in_area(&self, min: (i64, i64), max: (i64, i64)) -> Vec<Poi> {
        let mut found = Vec::new();
        // Each copy of the planet (a whole width east or west) the area
        // overlaps, then each chunk that might have a poi inside it.
        for copy in min.0.div_euclid(self.width)..=max.0.div_euclid(self.width) {
            let shift = copy * self.width;
            let west = (min.0 - shift).max(0);
            let east = (max.0 - shift).min(self.width - 1);
            if west > east {
                continue;
            }
            let (first, last) = (self.chunk_at(west, min.1), self.chunk_at(east, max.1));
            for cy in first.y..=last.y {
                for cx in first.x..=last.x {
                    let Some(mut poi) = self.in_chunk(ChunkPos { x: cx, y: cy }) else {
                        continue;
                    };
                    poi.x += shift;
                    if (min.0..=max.0).contains(&poi.x) && (min.1..=max.1).contains(&poi.y) {
                        found.push(poi);
                    }
                }
            }
        }
        found.sort_by_key(|poi| (poi.y, poi.x));
        found
    }
}
