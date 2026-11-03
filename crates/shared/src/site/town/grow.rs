//! Growing a town a plot at a time: beside, along and across from what is there.

use super::super::grid::{chance, roll};
use super::{
    BEHIND_FROM, BEHIND_PERCENT, Beside, LOOK_ALONG, Plot, SIDE_ROAD_AFTER, SIDE_ROAD_WEIGHT,
    SIDE_ROADS_FROM, Town, pick,
};

impl Town {
    /// Adds plots until the town has all its buildings or no room: each
    /// beside a plot from the list, more often an early one. One with nothing
    /// left beside it comes off the list.
    pub(super) fn grow(&mut self) {
        while (self.placed.len() as i64) < self.target {
            if self.plots.is_empty() {
                if !self.start_plot() {
                    return;
                }
                continue;
            }
            let last = self.plots.len() as i64 - 1;
            let index = roll(&mut self.rng, 0, last).min(roll(&mut self.rng, 0, last));
            let plot = self.plots[index as usize];
            if !self.add_beside(plot) {
                self.plots.remove(index as usize);
            }
        }
    }

    /// A plot on any road, as near the middle as there is room.
    fn start_plot(&mut self) -> bool {
        let (_, _, span, _) = self.level.bounds();
        for distance in 0..span {
            for road in 0..self.roads.len() {
                let middle = if self.roads[road].east_west {
                    self.middle.0
                } else {
                    self.middle.1
                };
                for side in [1, -1] {
                    for sign in [1, -1] {
                        let widest = self.widest();
                        let width = roll(&mut self.rng, 3, widest);
                        let a = middle + sign * distance;
                        if self.place_on(road, side, a, width, None) {
                            return true;
                        }
                    }
                }
            }
        }
        false
    }

    pub(super) fn widest(&self) -> i64 {
        if self.target <= 25 { 6 } else { 9 }
    }

    pub(super) fn deepest(&self) -> i64 {
        if self.target <= 25 { 6 } else { 8 }
    }

    /// Tries the ways of adding something beside a plot, likeliest first by
    /// chance, until one fits.
    fn add_beside(&mut self, plot: Plot) -> bool {
        let run = self.roads[plot.road].run;
        let corner = if self.target < SIDE_ROADS_FROM {
            0
        } else {
            2 + (run - SIDE_ROAD_AFTER).max(0) as u64 * SIDE_ROAD_WEIGHT
        };
        let behind_room = self.behind * 100 < self.target * BEHIND_PERCENT;
        let behind = if self.target >= BEHIND_FROM && behind_room {
            10
        } else {
            0
        };
        let mut ways = vec![
            (Beside::Along(1), 30),
            (Beside::Along(-1), 30),
            (Beside::Across, 30),
            (Beside::Corner(1), corner),
            (Beside::Corner(-1), corner),
            (Beside::Behind, behind),
        ];
        ways.retain(|&(_, weight)| weight > 0);
        while !ways.is_empty() {
            let choices: Vec<(usize, u64)> =
                ways.iter().enumerate().map(|(i, w)| (i, w.1)).collect();
            let (way, _) = ways.remove(pick(&mut self.rng, &choices));
            let done = match way {
                Beside::Along(sign) => self.add_along(plot, sign),
                Beside::Across => self.add_across(plot),
                Beside::Corner(sign) => self.add_corner(plot, sign),
                Beside::Behind => self.add_behind(plot),
            };
            if done {
                return true;
            }
        }
        false
    }

    /// Further along the road: touching, or with an alley left as bare ground
    /// (only a side road would make it more), its back lined up if it can be.
    fn add_along(&mut self, plot: Plot, sign: i64) -> bool {
        let gap = pick(&mut self.rng, &[(0, 55), (1, 30), (2, 15)]);
        let widest = self.widest();
        let width = roll(&mut self.rng, 3, widest);
        let deep = chance(&mut self.rng, 60).then_some(plot.deep);
        (0..=LOOK_ALONG).any(|extra| {
            let a = if sign > 0 {
                plot.along.1 + gap + extra
            } else {
                plot.along.0 - gap - extra - width
            };
            self.place_on(plot.road, plot.side, a, width, deep)
        })
    }

    /// Across the road, lined up with one end of the plot.
    fn add_across(&mut self, plot: Plot) -> bool {
        let widest = self.widest();
        let width = roll(&mut self.rng, 3, widest);
        let starts = [plot.along.0, plot.along.1 - width];
        let first = roll(&mut self.rng, 0, 1) as usize;
        [starts[first], starts[1 - first]]
            .into_iter()
            .any(|a| self.place_on(plot.road, -plot.side, a, width, None))
    }
}
