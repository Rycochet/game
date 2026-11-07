//! A Dev Area's one level: open ground inside a ring of wall, with its layout
//! up the middle and the way out on the west side.

use super::{DevArea, DevLayout};
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
        Self {
            level: make(&area.checked()),
        }
    }

    pub fn level(&self) -> SiteLevel {
        self.level.clone()
    }
}

/// Cells `0..size` each way inside, wall at `-1` and `size`.
fn make(area: &DevArea) -> SiteLevel {
    let size = i64::from(area.size);
    let mut level = Level::new(-1, -1, size + 2, size + 2);
    for y in 0..size {
        for x in 0..size {
            level.set(x, y, Cell::Ground);
        }
    }
    let middle = match area.layout {
        DevLayout::Empty => None,
        DevLayout::Moat => Some(Cell::Water),
        DevLayout::Cover => Some(Cell::Wall),
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
    let at = (STAIRS_IN, size / 2);
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
