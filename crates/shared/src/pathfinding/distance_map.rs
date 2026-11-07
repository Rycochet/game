//! How many steps every cell is from a start, up to a limit.

use std::cmp::Reverse;
use std::collections::BinaryHeap;

use super::DIRECTIONS;
use crate::dungeon::Level;
use crate::movement::{Step, mob_can_stand, mob_step_on_level, step_cost};

/// How long it takes to reach one goal from everywhere on a level, in
/// `movement::step_cost` units: for many mobs after one target, sharing the work.
/// By the mobs' rules (`movement::mob_step_on_level`): never over a World Gate.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DistanceMap {
    min: (i64, i64),
    width: i64,
    height: i64,
    costs: Vec<u32>,
}

impl DistanceMap {
    /// Flooded out from `goal` over legal steps.
    pub fn to(level: &Level, goal: (i64, i64)) -> Self {
        let (min_x, min_y, width, height) = level.bounds();
        let mut map = Self {
            min: (min_x, min_y),
            width,
            height,
            costs: vec![u32::MAX; (width * height) as usize],
        };
        let Some(start) = map.index(goal) else {
            return map;
        };
        map.costs[start] = 0;
        let mut queue = BinaryHeap::from([Reverse((0, goal))]);
        while let Some(Reverse((cost, cell))) = queue.pop() {
            if map.cost(cell).is_some_and(|known| known < cost) {
                continue;
            }
            // Whoever can step here, from one cell off or a jump away.
            for (dx, dy) in DIRECTIONS {
                for reach in [1, 2] {
                    let from = (cell.0 - dx * reach, cell.1 - dy * reach);
                    let step = Step::new(dx, dy).expect("a direction is a step");
                    if !mob_can_stand(level, from.0, from.1)
                        || mob_step_on_level(level, from, step) != Some(cell)
                    {
                        continue;
                    }
                    let total = cost + step_cost(from, cell);
                    if let Some(index) = map.index(from)
                        && total < map.costs[index]
                    {
                        map.costs[index] = total;
                        queue.push(Reverse((total, from)));
                    }
                }
            }
        }
        map
    }

    fn index(&self, (x, y): (i64, i64)) -> Option<usize> {
        let (col, row) = (x - self.min.0, y - self.min.1);
        ((0..self.width).contains(&col) && (0..self.height).contains(&row))
            .then(|| (row * self.width + col) as usize)
    }

    /// How long it takes from here: `None` if the goal can't be reached.
    pub fn cost(&self, cell: (i64, i64)) -> Option<u32> {
        let cost = self.costs[self.index(cell)?];
        (cost != u32::MAX).then_some(cost)
    }

    /// The legal step onto a `free` cell that leads to where it costs the least, if
    /// any is nearer than here.
    pub fn towards(
        &self,
        level: &Level,
        from: (i64, i64),
        free: impl Fn((i64, i64)) -> bool,
    ) -> Option<Step> {
        let here = self.cost(from).unwrap_or(u32::MAX);
        self.best_step(level, from, free, |cost| (cost < here).then_some(cost))
    }

    /// The legal step onto a `free` cell that leads furthest from the goal, if any
    /// is further than here. Somewhere the goal can't be reached from is furthest.
    pub fn away(
        &self,
        level: &Level,
        from: (i64, i64),
        free: impl Fn((i64, i64)) -> bool,
    ) -> Option<Step> {
        let here = self.cost(from).unwrap_or(u32::MAX);
        self.best_step(level, from, free, |cost| {
            (cost > here).then_some(u32::MAX - cost)
        })
    }

    /// The step whose landing scores lowest. A step that scores `None` isn't taken.
    fn best_step(
        &self,
        level: &Level,
        from: (i64, i64),
        free: impl Fn((i64, i64)) -> bool,
        score: impl Fn(u32) -> Option<u32>,
    ) -> Option<Step> {
        DIRECTIONS
            .iter()
            .filter_map(|&(dx, dy)| {
                let step = Step::new(dx, dy)?;
                let to = mob_step_on_level(level, from, step)?;
                if !free(to) {
                    return None;
                }
                Some((score(self.cost(to).unwrap_or(u32::MAX))?, step))
            })
            .min_by_key(|&(score, _)| score)
            .map(|(_, step)| step)
    }
}
