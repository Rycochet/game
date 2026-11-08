//! How much outside light spills to each cell, and the light from outside on every cell.

use crate::dungeon_map::liquid::LooksLiquid;
use bevy::prelude::*;
use shared::dungeon::{Cell, Level, line_is_clear};
use shared::protocol::SeenStairs;
use shared::site::BuildingLook;
use std::collections::BTreeSet;

use super::super::mesh::LightGrid;
use super::super::town::window_faces;
use super::OutsideLight;
use super::{
    FLASH_SPILL, LEAST_STRETCH, SPILL_REACH, SPILL_STRENGTH, STAIRS_REACH, STRETCH, WINDOW_REACH,
};

/// How much of the outside light spills to each cell, kept until the level or
/// the light changes: it costs too much to work out each time the camera moves.
#[derive(Default)]
pub(in crate::dungeon_map) struct Spill {
    min_x: i64,
    min_y: i64,
    width: i64,
    height: i64,
    amount: Vec<f32>,
}

/// Whether a cell has the sky over it. `open_air` says water and lava do too,
/// in places that are all in the open apart from their buildings; underground,
/// they don't.
pub(super) fn under_sky(cell: Cell, open_air: bool) -> bool {
    cell.is_outside() || (open_air && cell.looks_liquid())
}

impl Spill {
    /// Works it out for a level: from open cells beside anything open that isn't
    /// under the sky, stretched by `travel`, and down stairs open to the sky.
    /// Each cell takes the strongest reaching it, so more of them are no brighter.
    pub fn work_out(level: &Level, stairs: &[SeenStairs], travel: Vec2, open_air: bool) -> Self {
        let mut spill = Self::none(level);
        let (min_x, min_y, width, height) = level.bounds();
        let takes = |cell: Cell| !under_sky(cell, open_air);
        let sheltered =
            |x: i64, y: i64| level.is_open(x, y) && !under_sky(level.cell(x, y), open_air);
        for y in min_y..min_y + height {
            for x in min_x..min_x + width {
                let from_open_cell = under_sky(level.cell(x, y), open_air)
                    && (y - 1..=y + 1).any(|ny| (x - 1..=x + 1).any(|nx| sheltered(nx, ny)));
                if from_open_cell {
                    let stretch = |way: Vec2| (1.0 + STRETCH * way.dot(travel)).max(LEAST_STRETCH);
                    spill.add(level, (x, y), SPILL_REACH, 1.0 + STRETCH, takes, stretch);
                }
            }
        }
        for stair in stairs.iter().filter(|stair| stair.open_to_sky) {
            spill.add(level, (stair.x, stair.y), STAIRS_REACH, 1.0, takes, |_| 1.0);
        }
        spill
    }

    /// What the windows of the buildings in `on` give, the same way, from the
    /// cell in front of each, out in the open as much as anywhere.
    pub fn from_windows(level: &Level, buildings: &[BuildingLook], on: &BTreeSet<u16>) -> Self {
        let mut spill = Self::none(level);
        for building in buildings
            .iter()
            .filter(|building| on.contains(&building.id))
        {
            for rect in &building.rects {
                for y in rect.y..rect.y + rect.height {
                    for x in rect.x..rect.x + rect.width {
                        if level.cell(x, y) != Cell::BlindWindow {
                            continue;
                        }
                        if let Some((dx, dy)) = window_faces(level, (x, y)) {
                            let front = (x + dx, y + dy);
                            spill.add(level, front, WINDOW_REACH, 1.0, |_| true, |_| 1.0);
                        }
                    }
                }
            }
        }
        spill
    }

    /// Nothing anywhere on a level.
    fn none(level: &Level) -> Self {
        let (min_x, min_y, width, height) = level.bounds();
        Self {
            min_x,
            min_y,
            width,
            height,
            amount: vec![0.0; (width * height) as usize],
        }
    }

    fn index(&self, x: i64, y: i64) -> Option<usize> {
        let (col, row) = (x - self.min_x, y - self.min_y);
        ((0..self.width).contains(&col) && (0..self.height).contains(&row))
            .then_some((row * self.width + col) as usize)
    }

    /// Spills from one cell, as far as `reach` cells times `stretch` of the way
    /// to a cell (at most `longest`), stopped by walls, on the cells that `takes`
    /// it.
    fn add(
        &mut self,
        level: &Level,
        from: (i64, i64),
        reach: f32,
        longest: f32,
        takes: impl Fn(Cell) -> bool,
        stretch: impl Fn(Vec2) -> f32,
    ) {
        let far = (reach * longest).ceil() as i64;
        for y in from.1 - far..=from.1 + far {
            for x in from.0 - far..=from.0 + far {
                let Some(i) = self.index(x, y) else {
                    continue;
                };
                if !level.is_open(x, y) || !takes(level.cell(x, y)) {
                    continue;
                }
                let way = Vec2::new((x - from.0) as f32, (y - from.1) as f32);
                let distance = way.length() / stretch(way.normalize_or_zero());
                let fade = 1.0 - (distance / (reach + 1.0)).powi(2);
                let amount = SPILL_STRENGTH * fade;
                // The line last: it costs the most.
                if amount > self.amount[i] && line_is_clear(level, from, (x, y)) {
                    self.amount[i] = amount;
                }
            }
        }
    }

    pub fn at(&self, x: i64, y: i64) -> f32 {
        self.index(x, y).map_or(0.0, |i| self.amount[i])
    }
}

/// The light from outside on every cell: all of it under the sky, or in a town
/// (out in the open all over), and what spills in elsewhere.
pub(in crate::dungeon_map) fn light_from_outside(
    level: &Level,
    spill: &Spill,
    outside: &OutsideLight,
    flash: f32,
    town: bool,
    open_air: bool,
) -> LightGrid {
    let light = outside.in_flash(flash);
    if town {
        return LightGrid::filled(level, light);
    }
    // Lightning spills in further: as if many times as strong, never more than
    // what is outside.
    let spills = 1.0 + (FLASH_SPILL - 1.0) * flash;
    let mut grid = LightGrid::new(level);
    let (min_x, min_y, width, height) = level.bounds();
    for y in min_y..min_y + height {
        for x in min_x..min_x + width {
            if under_sky(level.cell(x, y), open_air) {
                grid.set(x, y, light);
            } else {
                grid.set(x, y, light * (spill.at(x, y) * spills).min(1.0));
            }
        }
    }
    grid
}
