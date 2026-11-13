//! What players are told of a wall: where it runs, what it is made of, its gates.

use serde::{Deserialize, Serialize};

use super::WallMaterial;
use crate::site::Rect;

/// A gate: two door cells side by side in a wall, `(x, y)` the west or north
/// one, the other east of it if the wall `runs_east_west`, else south.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Gate {
    pub x: i64,
    pub y: i64,
    pub runs_east_west: bool,
}

impl Gate {
    /// Its two cells.
    pub fn cells(self) -> [(i64, i64); 2] {
        if self.runs_east_west {
            [(self.x, self.y), (self.x + 1, self.y)]
        } else {
            [(self.x, self.y), (self.x, self.y + 1)]
        }
    }
}

/// A wall round a rectangle, as anyone can see it.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct WallLook {
    /// The rectangle the wall runs round, the wall's own cells included.
    pub rect: Rect,
    pub material: WallMaterial,
    pub gates: Vec<Gate>,
    /// Towers standing on it (a castle's): the wall stops at each and goes on
    /// past it, so none of their cells are the wall's.
    pub towers: Vec<Rect>,
    /// Cells of it fallen down (a ruin's): ground, not wall.
    pub gaps: Vec<(i64, i64)>,
}

impl WallLook {
    /// Whether `(x, y)` is one of the wall's cells (its gates' too), not in a
    /// tower or a gap.
    pub fn has(&self, x: i64, y: i64) -> bool {
        self.on_ring(x, y)
            && !self.towers.iter().any(|tower| tower.contains(x, y))
            && !self.gaps.contains(&(x, y))
    }

    /// Whether `(x, y)` is on the ring round its rectangle, towers and gaps
    /// or not.
    pub fn on_ring(&self, x: i64, y: i64) -> bool {
        let Rect {
            x: x0,
            y: y0,
            width,
            height,
        } = self.rect;
        let (x1, y1) = (x0 + width - 1, y0 + height - 1);
        let inside = (x0..=x1).contains(&x) && (y0..=y1).contains(&y);
        inside && (x == x0 || x == x1 || y == y0 || y == y1)
    }

    /// Whether `(x, y)` is in one of its gates.
    pub fn is_gate(&self, x: i64, y: i64) -> bool {
        self.gates.iter().any(|gate| gate.cells().contains(&(x, y)))
    }

    /// The same wall moved by `(dx, dy)`.
    pub fn moved(&self, (dx, dy): (i64, i64)) -> Self {
        Self {
            rect: Rect {
                x: self.rect.x + dx,
                y: self.rect.y + dy,
                ..self.rect
            },
            material: self.material,
            gates: (self.gates.iter())
                .map(|gate| Gate {
                    x: gate.x + dx,
                    y: gate.y + dy,
                    ..*gate
                })
                .collect(),
            towers: (self.towers.iter())
                .map(|tower| Rect {
                    x: tower.x + dx,
                    y: tower.y + dy,
                    ..*tower
                })
                .collect(),
            gaps: self.gaps.iter().map(|&(x, y)| (x + dx, y + dy)).collect(),
        }
    }
}
