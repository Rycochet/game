//! Wilderness cells against world map points, chunks and blocks.

use serde::{Deserialize, Serialize};

use super::Wilderness;
use crate::poi::{CHUNK_SIZE, ChunkPos};

/// Wilderness cells to a world map point, each way (owner). A poi stands at
/// least `poi::EDGE_MARGIN` points (128 cells) inside its chunk, so its site
/// fits in its own chunk.
pub const SCALE: i64 = 16;

/// Cells a side of a block, what the wilderness is made in. Chunks start on a
/// multiple of it, so a block is never in two chunks; the last block before the
/// planet's seam is narrower when the width isn't a multiple of it.
pub const BLOCK_SIZE: i64 = 64;

/// A block of the wilderness: cells `x * BLOCK_SIZE` on, `x` in
/// `0..Wilderness::block_columns`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct BlockPos {
    pub x: i64,
    pub y: i64,
}

impl Wilderness {
    /// Cell `x` brought into `0..width`.
    pub fn wrap(&self, x: i64) -> i64 {
        x.rem_euclid(self.width())
    }

    /// The world map point a cell is in.
    pub fn point_of(&self, x: i64, y: i64) -> (i64, i64) {
        (self.wrap(x).div_euclid(SCALE), y.div_euclid(SCALE))
    }

    /// The middle cell of a world map point: where exploring from it starts.
    pub fn centre_of(&self, px: i64, py: i64) -> (i64, i64) {
        (
            px.rem_euclid(self.points) * SCALE + SCALE / 2,
            py * SCALE + SCALE / 2,
        )
    }

    /// Chunk columns round the planet (the last one wider, as `PoiMap`'s).
    pub fn columns(&self) -> i64 {
        (self.points / CHUNK_SIZE).max(1)
    }

    /// The chunk a cell is in.
    pub fn chunk_of(&self, x: i64, y: i64) -> ChunkPos {
        let (px, py) = self.point_of(x, y);
        ChunkPos {
            x: (px / CHUNK_SIZE).min(self.columns() - 1),
            y: py.div_euclid(CHUNK_SIZE),
        }
    }

    /// A chunk's cells: `(min_x, min_y, width, height)`.
    pub fn chunk_bounds(&self, chunk: ChunkPos) -> (i64, i64, i64, i64) {
        let column = chunk.x.rem_euclid(self.columns());
        let west = column * CHUNK_SIZE;
        let east = if column == self.columns() - 1 {
            self.points
        } else {
            west + CHUNK_SIZE
        };
        let size = CHUNK_SIZE * SCALE;
        (west * SCALE, chunk.y * size, (east - west) * SCALE, size)
    }

    /// Blocks round the planet.
    pub fn block_columns(&self) -> i64 {
        (self.width() + BLOCK_SIZE - 1) / BLOCK_SIZE
    }

    /// The block a cell is in.
    pub fn block_of(&self, x: i64, y: i64) -> BlockPos {
        BlockPos {
            x: self.wrap(x) / BLOCK_SIZE,
            y: y.div_euclid(BLOCK_SIZE),
        }
    }

    /// A block's cells: `(min_x, min_y, width, height)`, `x` in `0..width`.
    pub fn block_bounds(&self, block: BlockPos) -> (i64, i64, i64, i64) {
        let x = block.x.rem_euclid(self.block_columns()) * BLOCK_SIZE;
        let width = BLOCK_SIZE.min(self.width() - x);
        (x, block.y * BLOCK_SIZE, width, BLOCK_SIZE)
    }
}
