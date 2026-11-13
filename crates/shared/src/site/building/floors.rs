//! A planned building's floors: the levels above ground, the basement and stilts.

use super::super::{Exit, PoiWall, SiteLevel, Stairs, StairsDirection};
use super::{Outline, Plan, Shape, add_windows, pillars};
use crate::dungeon::{Cell, Level};
use crate::rng::Seed;
use crate::site::Rect as CellRect;

impl Plan {
    /// Whether it has a basement, its last level.
    pub fn has_basement(&self) -> bool {
        self.basement.is_some()
    }

    /// The ground floor's outline, in level coordinates.
    pub(in crate::site) fn ground(&self) -> Shape {
        self.floors[0]
    }

    /// Each floor's width, from the ground up.
    #[cfg(test)]
    pub fn floor_widths(&self) -> Vec<i64> {
        self.floors.iter().map(|floor| floor.width).collect()
    }

    /// The ground floor's size in cells, without the border round it.
    pub fn footprint(&self) -> (i64, i64) {
        (self.floors[0].width, self.floors[0].height)
    }

    /// The wall round it as the world map shows it, if it has one.
    pub fn poi_wall(&self) -> Option<PoiWall> {
        let ground = self.floors[0];
        let inside = CellRect {
            x: ground.x,
            y: ground.y,
            width: ground.width,
            height: ground.height,
        };
        (self.wall.as_ref()).map(|wall| PoiWall::of(&wall.look, inside))
    }

    /// The shape of the ground floor.
    pub fn outline(&self) -> Outline {
        self.floors[0].outline
    }

    /// How many levels it has: its floors, and any basement.
    pub fn floors(&self) -> u32 {
        self.storeys() + u32::from(self.has_basement())
    }

    /// How many floors it has above the ground, and on it.
    fn storeys(&self) -> u32 {
        if self.ruined.is_empty() {
            self.floors.len() as u32
        } else {
            self.ruined.len() as u32
        }
    }

    /// Level `index` (0 the ground floor, then up, the last any basement).
    pub fn level(&self, index: u32) -> SiteLevel {
        if index >= self.storeys() {
            return self.basement_level();
        }
        let floor = match self.ruined.get(index as usize) {
            Some(floor) => floor.clone(),
            None => {
                let mut floor = self.intact_level(index);
                let shape = &self.floors[index as usize];
                if index > 0 || self.ground_windows {
                    add_windows(
                        &mut floor.level,
                        shape,
                        self.seed,
                        self.windows_percent,
                        i64::from(index),
                    );
                }
                floor
            }
        };
        match &self.wall {
            Some(wall) if index == 0 => wall.surround(self.seed, floor),
            _ => floor,
        }
    }

    /// The basement.
    fn basement_level(&self) -> SiteLevel {
        let (x, y) = self.basement.expect("a basement");
        let stairs = vec![Stairs {
            x,
            y,
            direction: StairsDirection::Up,
            to: Exit::Level(0),
        }];
        // (Not one of the floors' seeds: the floor count can change with ruin.)
        let rng = self.seed.derive("building basement").rng();
        self.fill_floor(self.basement_shape, rng, stairs, false)
    }

    /// Generates floor `index` as it is before any ruin.
    pub(super) fn intact_level(&self, index: u32) -> SiteLevel {
        let n = index as usize;
        let shape = self.floors[n];
        let rng = Seed::new(self.seed.derive("building floor").at(&[index as i64])).rng();

        // Stairs down to the floor below and up to the one above.
        let mut stairs = Vec::new();
        if let (0, Some((x, y))) = (n, self.basement) {
            stairs.push(Stairs {
                x,
                y,
                direction: StairsDirection::Down,
                to: Exit::Level(self.floors.len() as u32),
            });
        }
        if n > 0 {
            for &(x, y) in &self.stairs[n - 1] {
                stairs.push(Stairs {
                    x,
                    y,
                    direction: StairsDirection::Down,
                    to: Exit::Level(index - 1),
                });
            }
        }
        if n + 1 < self.floors.len() {
            for &(x, y) in &self.stairs[n] {
                stairs.push(Stairs {
                    x,
                    y,
                    direction: StairsDirection::Up,
                    to: Exit::Level(index + 1),
                });
            }
        }
        if n == 0 && self.stilts {
            return self.ground_under_stilts(shape, stairs);
        }
        self.fill_floor(shape, rng, stairs, n == 0)
    }

    /// The ground under a building on stilts: the outside ground, covered ground
    /// across the outline, and lone pillars round its edge. You arrive south of
    /// it.
    fn ground_under_stilts(&self, shape: Shape, stairs: Vec<Stairs>) -> SiteLevel {
        let mut level = Level::new(0, 0, self.width, self.height);
        for y in 0..self.height {
            for x in 0..self.width {
                let cell = if shape.contains(x, y) {
                    Cell::CoveredGround
                } else {
                    Cell::Ground
                };
                level.set(x, y, cell);
            }
        }
        for (x, y) in pillars(&shape) {
            level.set(x, y, Cell::Wall);
        }
        SiteLevel {
            level,
            stairs,
            entrance: Some((shape.x + shape.width / 2, shape.y + shape.height)),
            buildings: Vec::new(),
            fields: Vec::new(),
            walls: Vec::new(),
        }
    }
}
