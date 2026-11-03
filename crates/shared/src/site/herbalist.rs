//! Herbalists' huts: a small building of one storey over a basement that may
//! be bigger, with one herb patch outside: parallel strips of `Cell::HerbBed`
//! with bare ground to walk between them. What grows there is the client's to
//! choose; it is only looks.

use super::SiteLevel;
use super::building::{self, BuildingOptions};
use super::farm::{Plot, surroundings};
use super::grid::roll;
use crate::dungeon::{Cell, Level};
use crate::rng::Seed;

/// Options for the herbalist's hut generator.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct HerbalistOptions {
    pub house: BuildingOptions,
    /// How many strips the patch has, and how long they are, inclusive.
    pub strips: (i64, i64),
    pub length: (i64, i64),
}

/// A hut's plan. Its ground floor is made at once and kept, as a farm's is.
#[derive(Clone, Debug)]
pub struct Plan {
    house: building::Plan,
    ground: SiteLevel,
}

impl Plan {
    pub fn new(options: &HerbalistOptions, seed: Seed) -> Self {
        let house = building::Plan::new(&options.house, seed);
        let ground = lay_out(options, seed, &house);
        Self { house, ground }
    }

    pub fn house(&self) -> &building::Plan {
        &self.house
    }

    pub fn level(&self, index: u32) -> SiteLevel {
        match index {
            0 => self.ground.clone(),
            _ => self.house.level(index),
        }
    }
}

/// The ground floor with its patch, on a side of the hut away from its door.
fn lay_out(options: &HerbalistOptions, seed: Seed, house: &building::Plan) -> SiteLevel {
    let mut rng = seed.derive("herb patch").rng();
    let inner = house.level(0);
    let (_, _, width, height) = inner.level.bounds();
    let door = inner.entrance.expect("a hut has a way in");

    let strips = roll(&mut rng, options.strips.0, options.strips.1);
    let length = roll(&mut rng, options.length.0, options.length.1);
    let across = 2 * strips - 1;
    let east_west = rng.range_u64(0, 2) == 0;
    let (wide, tall) = if east_west {
        (length, across)
    } else {
        (across, length)
    };
    // The sides not nearest the door: right up against the yard round the hut.
    let by_door = |(dx, dy): (i64, i64)| {
        let (x, y) = (door.0 - width / 2, door.1 - height / 2);
        let (toward, other) = (x * dx + y * dy, (x * dy).abs() + (y * dx).abs());
        toward > other
    };
    let sides: Vec<(i64, i64)> = [(0, -1), (1, 0), (0, 1), (-1, 0)]
        .into_iter()
        .filter(|&side| !by_door(side))
        .collect();
    let side = sides[rng.range_u64(0, sides.len() as u64) as usize];
    let along = |rng: &mut _, room: i64, size: i64| roll(rng, -size / 2, room - size / 2);
    let x0 = match side.0 {
        -1 => -wide,
        1 => width,
        _ => along(&mut rng, width, wide),
    };
    let y0 = match side.1 {
        -1 => -tall,
        1 => height,
        _ => along(&mut rng, height, tall),
    };
    let plot = Plot {
        x0,
        y0,
        x1: x0 + wide,
        y1: y0 + tall,
    };

    let mut level: Level = surroundings(&inner.level, &[plot]);
    for y in plot.y0..plot.y1 {
        for x in plot.x0..plot.x1 {
            let strip = if east_west { y - plot.y0 } else { x - plot.x0 };
            if strip % 2 == 0 {
                level.set(x, y, Cell::HerbBed);
            }
        }
    }
    SiteLevel { level, ..inner }
}

#[cfg(test)]
mod tests;
