//! A cutout's looks in a wilderness window: towns' buildings, farms' fields and
//! forts' walls, turned into the window's frame.

use shared::site::{BuildingLook, FieldLook, Rect};

use super::window::Window;
use crate::wilderness::WildChunk;

/// Adds a chunk's cutout's looks to the window, `turn` cells across the seam.
pub(super) fn add_looks(window: &mut Window, wild_chunk: &WildChunk, turn: i64) {
    let turned = |rect: Rect| Rect {
        x: rect.x + turn,
        ..rect
    };
    for building in &wild_chunk.buildings {
        window.buildings.push(BuildingLook {
            rects: building.rects.iter().copied().map(turned).collect(),
            door: (building.door.0 + turn, building.door.1),
            ..building.clone()
        });
    }
    for field in &wild_chunk.fields {
        window.fields.push(FieldLook {
            rect: turned(field.rect),
            ..*field
        });
    }
    for wall in &wild_chunk.walls {
        window.walls.push(wall.moved((turn, 0)));
    }
}
