//! Helpers both generators use: random ranges, which cells can be reached,
//! joining unreached parts, and hiding some doors.

use crate::dungeon::{Cell, Level};
use crate::rng::Rng;

/// The four steps to a cell's neighbours (`y` south).
pub const STEPS: [(i64, i64); 4] = [(0, -1), (1, 0), (0, 1), (-1, 0)];

/// An integer in `min..=max`.
pub fn roll(rng: &mut Rng, min: i64, max: i64) -> i64 {
    if max <= min {
        return min;
    }
    min + rng.range_u64(0, (max - min + 1) as u64) as i64
}

/// True `percent` times in 100.
pub fn chance(rng: &mut Rng, percent: u32) -> bool {
    rng.range_u64(0, 100) < percent as u64
}

/// Which cells of a level can be walked to from some starting points.
pub struct Reach {
    min_x: i64,
    min_y: i64,
    width: i64,
    height: i64,
    reached: Vec<bool>,
}

impl Reach {
    /// Nothing reached yet.
    pub fn new(level: &Level) -> Self {
        let (min_x, min_y, width, height) = level.bounds();
        Self {
            min_x,
            min_y,
            width,
            height,
            reached: vec![false; (width * height) as usize],
        }
    }

    fn index(&self, x: i64, y: i64) -> Option<usize> {
        let (col, row) = (x - self.min_x, y - self.min_y);
        ((0..self.width).contains(&col) && (0..self.height).contains(&row))
            .then_some((row * self.width + col) as usize)
    }

    pub fn is(&self, x: i64, y: i64) -> bool {
        self.index(x, y).is_some_and(|i| self.reached[i])
    }

    /// Adds everything walkable that can be walked to from `from`
    /// (north, south, east and west steps).
    pub fn spread(&mut self, level: &Level, from: &[(i64, i64)]) {
        let mut stack: Vec<(i64, i64)> = Vec::new();
        for &(x, y) in from {
            if let Some(i) = self.index(x, y)
                && level.is_walkable(x, y)
                && !self.reached[i]
            {
                self.reached[i] = true;
                stack.push((x, y));
            }
        }
        while let Some((x, y)) = stack.pop() {
            for (dx, dy) in STEPS {
                let (nx, ny) = (x + dx, y + dy);
                if let Some(i) = self.index(nx, ny)
                    && !self.reached[i]
                    && level.is_walkable(nx, ny)
                {
                    self.reached[i] = true;
                    stack.push((nx, ny));
                }
            }
        }
    }

    /// The first walkable cell (row by row) that isn't reached.
    pub fn first_unreached(&self, level: &Level) -> Option<(i64, i64)> {
        (self.min_y..self.min_y + self.height)
            .flat_map(|y| (self.min_x..self.min_x + self.width).map(move |x| (x, y)))
            .find(|&(x, y)| level.is_walkable(x, y) && !self.is(x, y))
    }
}

/// Whether every walkable cell can be reached from `from`.
pub fn all_reached(level: &Level, from: &[(i64, i64)]) -> bool {
    let mut reach = Reach::new(level);
    reach.spread(level, from);
    reach.first_unreached(level).is_none()
}

/// Whether there is a door (open or secret) next to `(x, y)`, diagonals
/// included.
pub fn door_nearby(level: &Level, x: i64, y: i64) -> bool {
    (-1..=1).any(|dy| {
        (-1..=1).any(|dx| matches!(level.cell(x + dx, y + dy), Cell::Door | Cell::SecretDoor))
    })
}

/// Turns some doors (`percent` of them, at most) into secret doors, keeping
/// only those that nothing depends on: everything walkable can still be
/// reached from `from` without them. `keep(x, y)` marks doors to leave alone
/// (such as the way in).
pub fn hide_some_doors(
    level: &mut Level,
    rng: &mut Rng,
    percent: u32,
    from: &[(i64, i64)],
    keep: impl Fn(i64, i64) -> bool,
) {
    let (min_x, min_y, width, height) = level.bounds();
    for y in min_y..min_y + height {
        for x in min_x..min_x + width {
            if level.cell(x, y) != Cell::Door || keep(x, y) || !chance(rng, percent) {
                continue;
            }
            level.set(x, y, Cell::SecretDoor);
            if !all_reached(level, from) {
                level.set(x, y, Cell::Door);
            }
        }
    }
}

/// Digs the shortest way (through anything `diggable`) from the reached
/// cells to the nearest unreached walkable cell, as floor in no room.
/// Returns false if there is no way.
pub fn dig_to_unreached(
    level: &mut Level,
    reach: &Reach,
    diggable: impl Fn(i64, i64) -> bool,
) -> bool {
    let (min_x, min_y, width, height) = level.bounds();
    let index = |x: i64, y: i64| ((y - min_y) * width + (x - min_x)) as usize;
    let mut came_from: Vec<Option<(i64, i64)>> = vec![None; (width * height) as usize];
    let mut queue = std::collections::VecDeque::new();
    for y in min_y..min_y + height {
        for x in min_x..min_x + width {
            if reach.is(x, y) {
                came_from[index(x, y)] = Some((x, y));
                queue.push_back((x, y));
            }
        }
    }
    while let Some((x, y)) = queue.pop_front() {
        for (dx, dy) in STEPS {
            let (nx, ny) = (x + dx, y + dy);
            if !level.contains(nx, ny) || came_from[index(nx, ny)].is_some() {
                continue;
            }
            let walkable = level.is_walkable(nx, ny);
            if !walkable && !diggable(nx, ny) {
                continue;
            }
            came_from[index(nx, ny)] = Some((x, y));
            if walkable {
                // Found it: dig the way back.
                let mut at = (x, y);
                while !reach.is(at.0, at.1) {
                    if !level.is_walkable(at.0, at.1) {
                        level.set(at.0, at.1, Cell::Floor);
                        level.set_room(at.0, at.1, None);
                    }
                    at = came_from[index(at.0, at.1)].expect("a way back");
                }
                return true;
            }
            queue.push_back((nx, ny));
        }
    }
    false
}
