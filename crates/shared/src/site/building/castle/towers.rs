//! Where a castle's towers stand on its wall: on every corner, either side of
//! the gate, and spaced evenly along the runs between.

use serde::{Deserialize, Serialize};

use crate::site::Rect;

/// A tower's size across, both ways: those on the corners and along the wall,
/// and the short ones either side of the gate. Odd, so each is centred on the
/// wall.
pub const TOWER_SIZE: i64 = 7;
pub const GATE_TOWER_SIZE: i64 = 5;

/// Towers along a run of wall: one more for each this many cells between the
/// towers at its ends (and `RUN_SLACK` more), so they stand about this far
/// apart.
const RUN_PER_TOWER: i64 = 16;
const RUN_SLACK: i64 = 4;

/// Where one tower stands: its middle, on the wall, and its size.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct TowerSpot {
    pub x: i64,
    pub y: i64,
    pub size: i64,
    /// One of the two either side of the gate.
    pub by_gate: bool,
}

impl TowerSpot {
    /// The cells it stands on.
    pub fn rect(self) -> Rect {
        let half = self.size / 2;
        Rect {
            x: self.x - half,
            y: self.y - half,
            width: self.size,
            height: self.size,
        }
    }
}

/// The towers on a castle's wall round `rect` (the wall's own cells
/// included), whose gate is the two cells in the middle of the south side
/// (`gate_x` and the one east of it).
pub fn tower_spots(rect: Rect) -> Vec<TowerSpot> {
    let (x0, y0) = (rect.x, rect.y);
    let (x1, y1) = (x0 + rect.width - 1, y0 + rect.height - 1);
    let gate_x = gate_x(rect);
    let half = GATE_TOWER_SIZE / 2;
    let (gate_west, gate_east) = (gate_x - 1 - half, gate_x + 2 + half);
    let spot = |x, y, by_gate| TowerSpot {
        x,
        y,
        size: if by_gate { GATE_TOWER_SIZE } else { TOWER_SIZE },
        by_gate,
    };
    let mut spots = vec![
        spot(x0, y0, false),
        spot(x1, y0, false),
        spot(x1, y1, false),
        spot(x0, y1, false),
        spot(gate_west, y1, true),
        spot(gate_east, y1, true),
    ];
    // Runs between towers: north, east, the south either side of the gate,
    // west.
    let runs = [
        ((x0, y0), (x1, y0)),
        ((x1, y0), (x1, y1)),
        ((x0, y1), (gate_west, y1)),
        ((gate_east, y1), (x1, y1)),
        ((x0, y0), (x0, y1)),
    ];
    for ((ax, ay), (bx, by)) in runs {
        let length = (bx - ax).max(by - ay);
        let parts = ((length + RUN_SLACK) / RUN_PER_TOWER).max(1);
        for part in 1..parts {
            let along = length * part / parts;
            let (x, y) = if ay == by {
                (ax + along, ay)
            } else {
                (ax, ay + along)
            };
            spots.push(spot(x, y, false));
        }
    }
    spots
}

/// The west cell of the gate in the middle of the south side of a wall round
/// `rect`.
pub(in crate::site) fn gate_x(rect: Rect) -> i64 {
    rect.x + (rect.width - 2) / 2
}
