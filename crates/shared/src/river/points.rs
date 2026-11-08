//! Which world map points rivers run through, for walking: a step onto one
//! wades (`movement::world_step_cost`), and practises Swimming. Rivers are
//! traced a chunk at a time, so the points near where someone walks are
//! gathered as they go, on both sides alike.

use std::collections::BTreeSet;

use super::{MAX_LENGTH, Rivers};
use crate::poi::{CHUNK_SIZE, ChunkPos};

/// How many chunks each way round a point have their rivers traced: any
/// source close enough for its river to reach the point.
pub const RIVER_CHUNKS_ROUND: i64 = MAX_LENGTH as i64 / CHUNK_SIZE + 1;

/// The river points found so far, by where they are round the planet, and the
/// chunks already traced.
#[derive(Clone, Debug, Default)]
pub struct RiverPoints {
    width: i64,
    traced: BTreeSet<ChunkPos>,
    points: BTreeSet<(i64, i64)>,
}

impl RiverPoints {
    /// Traces every chunk within `RIVER_CHUNKS_ROUND` of `(x, y)` not traced
    /// yet: then every river point near it is known. Whether any were new.
    pub fn cover(&mut self, rivers: &Rivers, (x, y): (i64, i64)) -> bool {
        self.width = rivers.width();
        let columns = rivers.columns();
        let (cx, cy) = (
            x.rem_euclid(self.width.max(1))
                .div_euclid(CHUNK_SIZE)
                .min(columns - 1),
            y.div_euclid(CHUNK_SIZE),
        );
        let mut traced_any = false;
        for dy in -RIVER_CHUNKS_ROUND..=RIVER_CHUNKS_ROUND {
            for dx in -RIVER_CHUNKS_ROUND..=RIVER_CHUNKS_ROUND {
                let chunk = ChunkPos {
                    x: (cx + dx).rem_euclid(columns),
                    y: cy + dy,
                };
                if !self.traced.insert(chunk) {
                    continue;
                }
                traced_any = true;
                if let Some(river) = rivers.in_chunk(chunk) {
                    let width = self.width;
                    self.points.extend(
                        river
                            .points
                            .iter()
                            .map(|&(px, py)| (px.rem_euclid(width), py)),
                    );
                }
            }
        }
        traced_any
    }

    /// Whether a river runs through this point (as far as is known: `cover`
    /// round it first).
    pub fn contains(&self, (x, y): (i64, i64)) -> bool {
        self.width > 0 && self.points.contains(&(x.rem_euclid(self.width), y))
    }

    /// Whether no river point is known.
    pub fn is_empty(&self) -> bool {
        self.points.is_empty()
    }
}

#[cfg(test)]
mod tests;
