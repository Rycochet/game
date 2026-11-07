//! A Dev Area's one level: open ground inside a ring of wall, with its layout
//! in it and the way out on the west side (or at a building's own way in).

use super::{DevArea, DevLayout, biome, building};
use crate::dungeon::{Cell, Level};
use crate::site::{Exit, SiteLevel, Stairs, StairsDirection};

/// How far in from the west wall the stairs stand.
const STAIRS_IN: i64 = 1;

/// A Dev Area's plan: its level, made at once.
#[derive(Clone, Debug)]
pub struct Plan {
    level: SiteLevel,
}

impl Plan {
    pub fn new(area: &DevArea) -> Self {
        let area = area.checked();
        let level = match area.layout {
            DevLayout::Building(kind) => building::make(&area, kind),
            _ => plain(&area),
        };
        Self { level }
    }

    pub fn level(&self) -> SiteLevel {
        self.level.clone()
    }
}

/// Cells `0..size` each way inside, with the layout's middle or growth.
fn plain(area: &DevArea) -> SiteLevel {
    let size = i64::from(area.size);
    let mut level = walled((0, 0), size);
    let middle = match area.layout {
        DevLayout::Moat => Some(Cell::Water),
        DevLayout::Cover => Some(Cell::Wall),
        _ => None,
    };
    if let Some(cell) = middle {
        // A gap of an eighth of the side at each end.
        let gap = size / 8;
        for y in gap..size - gap {
            for x in size / 2 - 1..=size / 2 {
                level.set(x, y, cell);
            }
        }
    }
    if let DevLayout::Biome(kind) = area.layout {
        biome::grow(&mut level, area, kind);
    }
    let at = (STAIRS_IN, size / 2);
    clear_round(&mut level, at);
    with_stairs(level, at)
}

/// `size` × `size` outside ground with its north-west corner at `from`, in a
/// ring of wall.
pub(super) fn walled((x0, y0): (i64, i64), size: i64) -> Level {
    let mut level = Level::new(x0 - 1, y0 - 1, size + 2, size + 2);
    for y in y0..y0 + size {
        for x in x0..x0 + size {
            level.set(x, y, Cell::Ground);
        }
    }
    level
}

/// Open ground on and round the stairs, so nothing grows or stands on them.
pub(super) fn clear_round(level: &mut Level, (x, y): (i64, i64)) {
    for dy in -1..=1 {
        for dx in -1..=1 {
            let (cx, cy) = (x + dx, y + dy);
            // Never the ring of wall round the outside.
            let (min_x, min_y, width, height) = level.bounds();
            let inside =
                cx > min_x && cy > min_y && cx < min_x + width - 1 && cy < min_y + height - 1;
            if inside {
                level.set(cx, cy, Cell::Ground);
            }
        }
    }
}

/// The level, with the way out on `at`, which is also where to arrive.
pub(super) fn with_stairs(level: Level, at: (i64, i64)) -> SiteLevel {
    let stairs = Stairs {
        x: at.0,
        y: at.1,
        direction: StairsDirection::Up,
        to: Exit::WorldMap,
    };
    SiteLevel {
        level,
        stairs: vec![stairs],
        entrance: Some(at),
        buildings: Vec::new(),
        fields: Vec::new(),
    }
}
