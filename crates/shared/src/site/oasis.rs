//! Oases: open desert ground with one or two large pools, each ringed by
//! grass with trees growing on it (owner's design). One level, all outside.

use super::SiteLevel;
use super::grid::{Reach, roll};
use crate::dungeon::{Cell, Level};
use crate::rng::{Rng, Seed};

/// Options for the oasis generator.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct OasisOptions {
    /// The level's side, inclusive.
    pub size: (i64, i64),
    /// How many pools, and their radii across and down, inclusive.
    pub pools: (i64, i64),
    pub radius: (i64, i64),
    /// How wide the grass round a pool is, inclusive.
    pub grass: (i64, i64),
    /// Of the grass cells, how many in 100 grow a tree.
    pub trees_percent: u64,
}

/// An oasis's plan: its one level, made at once.
#[derive(Clone, Debug)]
pub struct Plan {
    ground: SiteLevel,
}

impl Plan {
    pub fn new(options: &OasisOptions, seed: Seed) -> Self {
        Self {
            ground: lay_out(options, seed),
        }
    }

    /// How many pools it has, for its look from the world map.
    pub fn pools(options: &OasisOptions, seed: Seed) -> u8 {
        let mut rng = seed.derive("oasis").rng();
        let _side = roll(&mut rng, options.size.0, options.size.1);
        roll(&mut rng, options.pools.0, options.pools.1) as u8
    }

    pub fn level(&self) -> SiteLevel {
        self.ground.clone()
    }
}

/// One pool: its centre and radii, as an ellipse.
struct Pool {
    centre: (i64, i64),
    radius: (i64, i64),
}

impl Pool {
    /// How far out a cell is, squared and scaled: 1 on the water's edge.
    fn reach(&self, (x, y): (i64, i64), extra: i64) -> f64 {
        let dx = (x - self.centre.0) as f64 / (self.radius.0 + extra) as f64;
        let dy = (y - self.centre.1) as f64 / (self.radius.1 + extra) as f64;
        dx * dx + dy * dy
    }
}

fn lay_out(options: &OasisOptions, seed: Seed) -> SiteLevel {
    let mut rng = seed.derive("oasis").rng();
    let side = roll(&mut rng, options.size.0, options.size.1);
    let count = roll(&mut rng, options.pools.0, options.pools.1);
    let grass = roll(&mut rng, options.grass.0, options.grass.1);
    let mut level = Level::new(0, 0, side, side);
    for y in 0..side {
        for x in 0..side {
            level.set(x, y, Cell::Ground);
        }
    }
    let pools: Vec<Pool> = (0..count)
        .map(|_| pool(&mut rng, options, side, grass))
        .collect();
    for y in 0..side {
        for x in 0..side {
            if pools.iter().any(|pool| pool.reach((x, y), 0) <= 1.0) {
                level.set(x, y, Cell::Water);
            } else if pools.iter().any(|pool| pool.reach((x, y), grass) <= 1.0) {
                let tree = seed.derive("trees").at(&[x, y]) % 100 < options.trees_percent;
                level.set(x, y, if tree { Cell::Tree } else { Cell::Grass });
            }
        }
    }
    let entrance = (side / 2, side - 1);
    level.set(entrance.0, entrance.1, Cell::Ground);
    open_up(&mut level, entrance);
    SiteLevel {
        level,
        stairs: Vec::new(),
        entrance: Some(entrance),
        buildings: Vec::new(),
        fields: Vec::new(),
    }
}

/// A pool somewhere in the middle, with room round it for its grass and a
/// walk round that.
fn pool(rng: &mut Rng, options: &OasisOptions, side: i64, grass: i64) -> Pool {
    let radius = (
        roll(rng, options.radius.0, options.radius.1),
        roll(rng, options.radius.0, options.radius.1),
    );
    let margin = |r: i64| (r + grass + 2).min(side / 2);
    let centre = (
        roll(rng, margin(radius.0), side - 1 - margin(radius.0)),
        roll(rng, margin(radius.1), side - 1 - margin(radius.1)),
    );
    Pool { centre, radius }
}

/// Makes everything walkable reachable from the way in: trees shutting
/// ground off fall to grass, one at a time, and what is still cut off (grass
/// on an island) becomes water.
fn open_up(level: &mut Level, entrance: (i64, i64)) {
    let (_, _, width, height) = level.bounds();
    let mut reach = Reach::new(level);
    reach.spread(level, &[entrance]);
    while reach.first_unreached(level).is_some() {
        let unreached = |reach: &Reach, level: &Level, (x, y): (i64, i64)| {
            level.is_walkable(x, y) && !reach.is(x, y)
        };
        let blocking = (0..height)
            .flat_map(|y| (0..width).map(move |x| (x, y)))
            .find(|&(x, y)| {
                level.cell(x, y) == Cell::Tree
                    && STEPS.iter().any(|(dx, dy)| reach.is(x + dx, y + dy))
                    && STEPS
                        .iter()
                        .any(|(dx, dy)| unreached(&reach, level, (x + dx, y + dy)))
            });
        let Some((x, y)) = blocking else {
            while let Some((x, y)) = reach.first_unreached(level) {
                level.set(x, y, Cell::Water);
            }
            return;
        };
        level.set(x, y, Cell::Grass);
        reach.spread(level, &[(x, y)]);
    }
}

const STEPS: [(i64, i64); 4] = [(0, -1), (1, 0), (0, 1), (-1, 0)];

#[cfg(test)]
mod tests;
