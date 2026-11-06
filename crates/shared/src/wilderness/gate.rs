//! A World Gate standing in the wilderness: open ground round the cell its
//! frame stands on, which is the way into the Hall of Worlds.

use super::Wilderness;
use crate::world_gate::WorldGate;

/// How far round a gate's cell the ground is open, in cells.
pub const GATE_PAD: i64 = 1;

/// A World Gate where it stands in the wilderness.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct GateSpot {
    /// Which of the world's gates (`hall::GateAddress::entrance`).
    pub entrance: u8,
    /// The cell its frame stands on: `Cell::WorldGate`, the middle cell of its
    /// world map point.
    pub cell: (i64, i64),
}

impl GateSpot {
    /// The gate `entrance` of the world, at `gate`.
    pub fn new(wild: &Wilderness, entrance: u8, gate: &WorldGate) -> Self {
        Self {
            entrance,
            cell: wild.centre_of(gate.x, gate.y),
        }
    }

    /// Whether a cell is on its open ground (the gate's cell included).
    pub fn contains(&self, x: i64, y: i64) -> bool {
        (x - self.cell.0).abs() <= GATE_PAD && (y - self.cell.1).abs() <= GATE_PAD
    }

    /// Whether its open ground comes within `margin` cells of a rectangle of
    /// cells, on any copy of the planet.
    pub fn near(
        &self,
        wild: &Wilderness,
        (min_x, min_y, width, height): (i64, i64, i64, i64),
        margin: i64,
    ) -> bool {
        let reach = GATE_PAD + margin;
        let (x, y) = self.cell;
        [-wild.width(), 0, wild.width()].iter().any(|shift| {
            x + shift + reach >= min_x
                && x + shift - reach < min_x + width
                && y + reach >= min_y
                && y - reach < min_y + height
        })
    }

    /// The square of the distance in cells from a cell to its open ground.
    pub(super) fn distance_squared(&self, x: i64, y: i64) -> i64 {
        let dx = ((x - self.cell.0).abs() - GATE_PAD).max(0);
        let dy = ((y - self.cell.1).abs() - GATE_PAD).max(0);
        dx * dx + dy * dy
    }
}
