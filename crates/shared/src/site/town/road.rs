//! Roads and the blocks of cells they cover.

use super::Frame;

/// A rectangle of cells (`x1`, `y1` just beyond it).
#[derive(Clone, Copy, Debug)]
pub(super) struct Block {
    pub(super) x0: i64,
    pub(super) y0: i64,
    pub(super) x1: i64,
    pub(super) y1: i64,
}

impl Block {
    pub(super) fn width(self) -> i64 {
        self.x1 - self.x0
    }
}

/// A straight road buildings stand along.
#[derive(Clone, Copy, Debug)]
pub(super) struct Road {
    pub(super) east_west: bool,
    /// The rows (or columns) it covers.
    pub(super) across: (i64, i64),
    /// How far it goes, from the first up to just before the second.
    pub(super) along: (i64, i64),
    /// An index into `ROADS`.
    pub(super) kind: usize,
    /// Whether it stays as it is: main roads and a square's edges.
    pub(super) fixed: bool,
    /// Tiles of frontage since its last side road.
    pub(super) run: i64,
}

impl Road {
    /// The cells it covers from `from` up to just before `to`.
    pub(super) fn area(&self, from: i64, to: i64) -> Block {
        if self.east_west {
            Block {
                x0: from,
                x1: to,
                y0: self.across.0,
                y1: self.across.1,
            }
        } else {
            Block {
                x0: self.across.0,
                x1: self.across.1,
                y0: from,
                y1: to,
            }
        }
    }

    /// A plot starting `a` along it, on its side `side` (1 is south or east,
    /// -1 north or west).
    pub(super) fn frame(&self, side: i64, a: i64) -> Frame {
        let edge = if side > 0 {
            self.across.1
        } else {
            self.across.0 - 1
        };
        if self.east_west {
            Frame {
                corner: (a, edge),
                along: (1, 0),
                inward: (0, side),
            }
        } else {
            Frame {
                corner: (edge, a),
                along: (0, 1),
                inward: (side, 0),
            }
        }
    }
}
