//! The ways into a cutout from the land: town roads reaching its edge.

use super::{MEET, Point};
use crate::dungeon::Level;

/// One edge's cells in order, the way out from it, and where on a cell the
/// middle of its outer side is.
type Side = (Vec<(i64, i64)>, Point, Point);

/// A way into a cutout: the middle of a run of road cells on its edge, on
/// the edge's outer side, and the way out from it.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) struct WayIn {
    pub(super) at: Point,
    pub(super) out: Point,
}

impl WayIn {
    /// Where roads coming to it meet, straight out from it.
    pub(super) fn meet(&self) -> Point {
        (self.at.0 + self.out.0 * MEET, self.at.1 + self.out.1 * MEET)
    }
}

/// The ways into a level from outside: every run of road cells along each of
/// its edges, north, east, south and west, each run west to east or north to
/// south.
pub(super) fn ways_in(level: &Level) -> Vec<WayIn> {
    let (x0, y0, width, height) = level.bounds();
    let (x1, y1) = (x0 + width - 1, y0 + height - 1);
    let sides: [Side; 4] = [
        (
            (x0..=x1).map(|x| (x, y0)).collect(),
            (0.0, -1.0),
            (0.5, 0.0),
        ),
        ((y0..=y1).map(|y| (x1, y)).collect(), (1.0, 0.0), (1.0, 0.5)),
        ((x0..=x1).map(|x| (x, y1)).collect(), (0.0, 1.0), (0.5, 1.0)),
        (
            (y0..=y1).map(|y| (x0, y)).collect(),
            (-1.0, 0.0),
            (0.0, 0.5),
        ),
    ];
    let mut ways = Vec::new();
    for (cells, out, corner) in sides {
        let mut run: Vec<(i64, i64)> = Vec::new();
        for cell in cells.iter().copied().chain([(i64::MIN, i64::MIN)]) {
            let road = cell.0 != i64::MIN && level.cell(cell.0, cell.1).is_road();
            if road {
                run.push(cell);
                continue;
            }
            if let (Some(first), Some(last)) = (run.first(), run.last()) {
                // The middle of the run's outer side.
                let at = (
                    (first.0 + last.0) as f64 / 2.0 + corner.0,
                    (first.1 + last.1) as f64 / 2.0 + corner.1,
                );
                ways.push(WayIn { at, out });
            }
            run.clear();
        }
    }
    ways
}
