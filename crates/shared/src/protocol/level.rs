//! Site levels as the server sends them: what a level is, then its cells in
//! 8×8 chunks, only as the player comes to see them (or is about to).

use serde::{Deserialize, Serialize};

use crate::dungeon::Cell;
use crate::poi::PoiKind;
use crate::site::{StairsDirection, StairsStyle};

/// Level chunks are this many cells on a side: 64 cells, one bit each of a
/// `u64` mask.
pub const LEVEL_CHUNK_SIZE: i64 = 8;

/// How close (each way, in world map grid points) the player must be to a
/// poi to enter it.
pub const ENTER_REACH: i64 = 2;

/// The chunk a cell is in, and its bit in that chunk's mask.
pub fn level_chunk(x: i64, y: i64) -> ((i64, i64), u32) {
    let chunk = (
        x.div_euclid(LEVEL_CHUNK_SIZE),
        y.div_euclid(LEVEL_CHUNK_SIZE),
    );
    let bit = y.rem_euclid(LEVEL_CHUNK_SIZE) * LEVEL_CHUNK_SIZE + x.rem_euclid(LEVEL_CHUNK_SIZE);
    (chunk, bit as u32)
}

/// What a player is told on arriving on a site level: everything except its
/// cells, which follow as `LevelChunk`s.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct LevelInfo {
    /// The kind of place (for its look and name).
    pub kind: PoiKind,
    /// Whether its levels are a building's floors (going up) rather than a
    /// cave's depths.
    pub building: bool,
    /// Which level this is, of how many.
    pub index: u32,
    pub levels: u32,
    /// The level's rectangle: `(min_x, min_y, width, height)`.
    pub bounds: (i64, i64, i64, i64),
    /// Where the player arrives.
    pub arrival: (i64, i64),
}

/// Some cells of one chunk of the current level.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct LevelChunk {
    /// Which chunk (cell `x, y` is in chunk `x / 8, y / 8`, rounding down).
    pub chunk: (i64, i64),
    /// Which of its cells are included: bit `row × 8 + column`.
    pub mask: u64,
    /// Those cells, in bit order. What the player would see: a secret door
    /// is a wall.
    pub cells: Vec<Cell>,
    /// Stairs on those cells.
    pub stairs: Vec<SeenStairs>,
    /// Whether these are cells the player saw on an earlier visit (sent on
    /// arriving), rather than ones they are about to see.
    pub remembered: bool,
}

impl LevelChunk {
    /// The positions of the cells the mask includes, in bit order.
    pub fn positions(&self) -> impl Iterator<Item = (i64, i64)> + '_ {
        let (cx, cy) = self.chunk;
        (0..64i64)
            .filter(|bit| self.mask & (1 << bit) != 0)
            .map(move |bit| {
                (
                    cx * LEVEL_CHUNK_SIZE + bit % LEVEL_CHUNK_SIZE,
                    cy * LEVEL_CHUNK_SIZE + bit / LEVEL_CHUNK_SIZE,
                )
            })
    }

    /// The included cells with their positions, in bit order.
    pub fn cells(&self) -> impl Iterator<Item = (i64, i64, Cell)> + '_ {
        self.positions()
            .zip(&self.cells)
            .map(|((x, y), &cell)| (x, y, cell))
    }
}

/// A staircase as the player sees it: where it leads stays with the server.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct SeenStairs {
    pub x: i64,
    pub y: i64,
    pub direction: StairsDirection,
    /// The way it goes down, as a grid step from its top end to its bottom
    /// end (`Site::stairs_descend`), the same on both levels it joins.
    pub descends: (i64, i64),
    /// What it is built as (`Site::stairs_style`), the same at both ends.
    pub style: StairsStyle,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cells_come_back_where_they_were() {
        let cells = [(-1, -1), (-8, -8), (-3, -6)];
        let mut mask = 0;
        for (x, y) in cells {
            let (chunk, bit) = level_chunk(x, y);
            assert_eq!(chunk, (-1, -1));
            mask |= 1 << bit;
        }
        let chunk = LevelChunk {
            chunk: (-1, -1),
            mask,
            cells: vec![Cell::Floor, Cell::Wall, Cell::Water],
            stairs: Vec::new(),
            remembered: false,
        };
        let back: Vec<_> = chunk.cells().map(|(x, y, _)| (x, y)).collect();
        // Bit order: north-west first, row by row.
        assert_eq!(back, [(-8, -8), (-3, -6), (-1, -1)]);
        assert_eq!(level_chunk(0, 0), ((0, 0), 0));
        assert_eq!(level_chunk(7, 7), ((0, 0), 63));
    }
}
