//! A wall as the world map shows it.

use serde::{Deserialize, Serialize};

use super::{WallLook, WallMaterial};
use crate::site::Rect;

/// A wall round a building, as seen from the world map (`PoiLook::Building`):
/// the ground between the building's footprint and the wall on each side, in
/// cells, what it is made of, and which sides have a gate in their middle.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct PoiWall {
    pub west: u8,
    pub north: u8,
    pub east: u8,
    pub south: u8,
    pub material: WallMaterial,
    /// North, east, south and west.
    pub gates: [bool; 4],
}

impl PoiWall {
    /// The world map's look of `wall`, round a building's footprint `inside`.
    pub fn of(wall: &WallLook, inside: Rect) -> Self {
        let rect = wall.rect;
        let gap = |cells: i64| cells.clamp(0, i64::from(u8::MAX)) as u8;
        let (x1, y1) = (rect.x + rect.width - 1, rect.y + rect.height - 1);
        let mut gates = [false; 4];
        for gate in &wall.gates {
            let side = match (gate.runs_east_west, gate.y == rect.y, gate.x == rect.x) {
                (true, true, _) => 0,
                (true, false, _) => 2,
                (false, _, true) => 3,
                (false, _, false) => 1,
            };
            gates[side] = true;
        }
        Self {
            west: gap(inside.x - rect.x - 1),
            north: gap(inside.y - rect.y - 1),
            east: gap(x1 - (inside.x + inside.width)),
            south: gap(y1 - (inside.y + inside.height)),
            material: wall.material,
            gates,
        }
    }
}
