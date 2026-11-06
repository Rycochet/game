//! A chunk's poi set into the wilderness: its site's first level, or only the
//! way down into it.

use super::{CLEARED_TO, ENTRANCE_CLEARING, Wilderness};
use crate::dungeon::{Cell, Level};
use crate::poi::Poi;
use crate::protocol::LEVEL_CHUNK_SIZE;
use crate::site::{Exit, SiteLevel, Stairs, StairsDirection};

/// Where a poi's site meets the wilderness.
#[derive(Clone, Debug)]
pub struct Cutout {
    poi: Poi,
    /// Added to a cell of the site's first level to give the wilderness cell
    /// it is set at; `None` when only the way down is (a site entered from
    /// above, such as a cave by a hole).
    offset: Option<(i64, i64)>,
    /// The cells set into the wilderness, in wilderness cells.
    level: Level,
    /// The ways from the cutout into the site's levels, in wilderness cells.
    stairs: Vec<Stairs>,
}

impl Cutout {
    /// The poi's cutout, from its site's first level, kept inside its chunk
    /// with room for the clearing round it.
    pub fn new(wild: &Wilderness, poi: Poi, first: &SiteLevel) -> Self {
        let (cx, cy) = wild.centre_of(poi.x, poi.y);
        let from_above = first
            .stairs
            .iter()
            .any(|stairs| stairs.to == Exit::WorldMap);
        if from_above {
            let side = 2 * ENTRANCE_CLEARING + 1;
            let mut level = Level::new(cx - ENTRANCE_CLEARING, cy - ENTRANCE_CLEARING, side, side);
            for y in cy - ENTRANCE_CLEARING..=cy + ENTRANCE_CLEARING {
                for x in cx - ENTRANCE_CLEARING..=cx + ENTRANCE_CLEARING {
                    level.set(x, y, Cell::Ground);
                }
            }
            let stairs = Stairs {
                x: cx,
                y: cy,
                direction: StairsDirection::Down,
                to: Exit::Level(0),
            };
            return Self {
                poi,
                offset: None,
                level,
                stairs: vec![stairs],
            };
        }
        let (lx, ly, width, height) = first.level.bounds();
        let (min_x, min_y, chunk_width, chunk_height) = wild.chunk_bounds(poi.chunk);
        let place = |centre: i64, low: i64, size: i64, chunk_min: i64, chunk_size: i64| {
            let wanted = round_to_chunk(centre - size / 2 - low);
            let least = ceil_to_chunk(chunk_min + CLEARED_TO - low);
            let most = floor_to_chunk(chunk_min + chunk_size - CLEARED_TO - size - low);
            wanted.min(most).max(least)
        };
        let ox = place(cx, lx, width, min_x, chunk_width);
        let oy = place(cy, ly, height, min_y, chunk_height);
        let mut level = Level::new(lx + ox, ly + oy, width, height);
        for y in ly..ly + height {
            for x in lx..lx + width {
                level.set(x + ox, y + oy, first.level.cell(x, y));
                level.set_room(x + ox, y + oy, first.level.room(x, y));
            }
        }
        let stairs = first
            .stairs
            .iter()
            .map(|&stairs| Stairs {
                x: stairs.x + ox,
                y: stairs.y + oy,
                ..stairs
            })
            .collect();
        Self {
            poi,
            offset: Some((ox, oy)),
            level,
            stairs,
        }
    }

    pub fn poi(&self) -> &Poi {
        &self.poi
    }

    /// Added to a cell of the site's first level to give its wilderness cell
    /// (a multiple of `LEVEL_CHUNK_SIZE` each way, so the site's cell chunks
    /// are the wilderness's); `None` if only the way down is set in.
    pub fn offset(&self) -> Option<(i64, i64)> {
        self.offset
    }

    /// The cells set into the wilderness, in wilderness cells.
    pub fn level(&self) -> &Level {
        &self.level
    }

    /// The ways from the cutout into the site's levels, in wilderness cells.
    pub fn stairs(&self) -> &[Stairs] {
        &self.stairs
    }

    /// Whether a wilderness cell is the cutout's.
    pub fn contains(&self, x: i64, y: i64) -> bool {
        self.level.contains(x, y)
    }

    /// The square of the distance in cells from a cell to the cutout, 0 inside.
    pub(super) fn distance_squared(&self, x: i64, y: i64) -> i64 {
        let (min_x, min_y, width, height) = self.level.bounds();
        let dx = (min_x - x).max(x - (min_x + width - 1)).max(0);
        let dy = (min_y - y).max(y - (min_y + height - 1)).max(0);
        dx * dx + dy * dy
    }
}

const STEP: i64 = LEVEL_CHUNK_SIZE;

fn floor_to_chunk(value: i64) -> i64 {
    value.div_euclid(STEP) * STEP
}

fn ceil_to_chunk(value: i64) -> i64 {
    floor_to_chunk(value + STEP - 1)
}

fn round_to_chunk(value: i64) -> i64 {
    floor_to_chunk(value + STEP / 2)
}
